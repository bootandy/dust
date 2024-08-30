mod cli;
mod config;
mod dir_walker;
mod display;
mod display_node;
mod filter;
mod filter_type;
mod node;
mod platform;
mod progress;
mod utils;

use crate::cli::build_cli;
use crate::progress::RuntimeErrors;
use clap::parser::ValuesRef;
use dir_walker::WalkData;
use display::InitialDisplayData;
use filter::AggregateData;
use progress::PIndicator;
use regex::Error;
use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::io;
use std::panic;
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use sysinfo::{System, SystemExt};

use self::display::draw_it;
use config::get_config;
use dir_walker::walk_it;
use filter::get_biggest;
use filter_type::get_all_file_types;
use regex::Regex;
use std::cmp::max;
use std::path::PathBuf;
use terminal_size::{terminal_size, Height, Width};
use utils::get_filesystem_devices;
use utils::simplify_dir_names;

static DEFAULT_NUMBER_OF_LINES: usize = 30;
static DEFAULT_TERMINAL_WIDTH: usize = 80;

fn should_init_color(no_color: bool, force_color: bool) -> bool {
    if force_color {
        return true;
    }
    if no_color {
        return false;
    }
    // check if NO_COLOR is set
    // https://no-color.org/
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if terminal_size().is_none() {
        // we are not in a terminal, color may not be needed
        return false;
    }
    // we are in a terminal
    #[cfg(windows)]
    {
        // Required for windows 10
        // Fails to resolve for windows 8 so disable color
        match ansi_term::enable_ansi_support() {
            Ok(_) => true,
            Err(_) => {
                eprintln!("This version of Windows does not support ANSI colors");
                false
            }
        }
    }
    #[cfg(not(windows))]
    {
        true
    }
}

fn get_height_of_terminal() -> usize {
    terminal_size()
        // Windows CI runners detect a terminal height of 0
        .map(|(_, Height(h))| max(h.into(), DEFAULT_NUMBER_OF_LINES))
        .unwrap_or(DEFAULT_NUMBER_OF_LINES)
        - 10
}

fn get_width_of_terminal() -> usize {
    terminal_size()
        .map(|(Width(w), _)| match cfg!(windows) {
            // Windows CI runners detect a very low terminal width
            true => max(w.into(), DEFAULT_TERMINAL_WIDTH),
            false => w.into(),
        })
        .unwrap_or(DEFAULT_TERMINAL_WIDTH)
}

fn get_regex_value(maybe_value: Option<ValuesRef<String>>) -> Vec<Regex> {
    maybe_value
        .unwrap_or_default()
        .map(|reg| {
            Regex::new(reg).unwrap_or_else(|err| {
                eprintln!("Ignoring bad value for regex {err:?}");
                process::exit(1)
            })
        })
        .collect()
}

fn main() {
    let options = build_cli().get_matches();
    let config = get_config();

    let errors = RuntimeErrors::default();
    let error_listen_for_ctrlc = Arc::new(Mutex::new(errors));
    let errors_for_rayon = error_listen_for_ctrlc.clone();
    let errors_final = error_listen_for_ctrlc.clone();
    let is_in_listing = Arc::new(AtomicBool::new(false));
    let cloned_is_in_listing = Arc::clone(&is_in_listing);

    ctrlc::set_handler(move || {
        error_listen_for_ctrlc.lock().unwrap().abort = true;
        println!("\nAborting");
        if cloned_is_in_listing.load(Ordering::Relaxed) {
            process::exit(1);
        }
    })
    .expect("Error setting Ctrl-C handler");

    is_in_listing.store(true, Ordering::Relaxed);
    let target_dirs = match config.get_files_from(&options) {
        Some(path) => {
            if path == "-" {
                let mut targets_to_add = io::stdin()
                    .lines()
                    .map_while(Result::ok)
                    .collect::<Vec<String>>();

                if targets_to_add.is_empty() {
                    eprintln!("No input provided, defaulting to current directory");
                    targets_to_add.push(".".to_owned());
                }
                targets_to_add
            } else {
                // read file
                match read_to_string(path) {
                    Ok(file_content) => file_content.lines().map(|x| x.to_string()).collect(),
                    Err(e) => {
                        eprintln!("Error reading file: {e}");
                        vec![".".to_owned()]
                    }
                }
            }
        }
        None => match options.get_many::<String>("params") {
            Some(values) => values.cloned().collect(),
            None => vec![".".to_owned()],
        },
    };
    is_in_listing.store(false, Ordering::Relaxed);

    let summarize_file_types = options.get_flag("types");

    let filter_regexs = get_regex_value(options.get_many("filter"));
    let invert_filter_regexs = get_regex_value(options.get_many("invert_filter"));

    let terminal_width: usize = match options.get_one::<usize>("width") {
        Some(&val) => val,
        None => get_width_of_terminal(),
    };

    let depth = config.get_depth(&options);

    // If depth is set, then we set the default number_of_lines to be max
    // instead of screen height

    let number_of_lines = match options.get_one::<usize>("number_of_lines") {
        Some(&val) => val,
        None => {
            if depth != usize::MAX {
                usize::MAX
            } else {
                get_height_of_terminal()
            }
        }
    };

    let is_colors = should_init_color(
        config.get_no_colors(&options),
        config.get_force_colors(&options),
    );

    let ignore_directories = match options.get_many::<String>("ignore_directory") {
        Some(values) => values
            .map(|v| v.as_str())
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>(),
        None => vec![],
    };

    let ignore_from_file_result = match options.get_one::<String>("ignore_all_in_file") {
        Some(val) => read_to_string(val)
            .unwrap()
            .lines()
            .map(Regex::new)
            .collect::<Vec<Result<Regex, Error>>>(),
        None => vec![],
    };
    let ignore_from_file = ignore_from_file_result
        .into_iter()
        .filter_map(|x| x.ok())
        .collect::<Vec<Regex>>();

    let invert_filter_regexs = invert_filter_regexs
        .into_iter()
        .chain(ignore_from_file)
        .collect::<Vec<Regex>>();

    let by_filecount = options.get_flag("by_filecount");
    let by_filetime = config.get_filetime(&options);
    let limit_filesystem = options.get_flag("limit_filesystem");
    let follow_links = options.get_flag("dereference_links");

    let allowed_filesystems = limit_filesystem
        .then(|| get_filesystem_devices(&target_dirs, follow_links))
        .unwrap_or_default();
    let simplified_dirs = simplify_dir_names(&target_dirs);

    let ignored_full_path: HashSet<PathBuf> = ignore_directories
        .into_iter()
        .flat_map(|x| simplified_dirs.iter().map(move |d| d.join(&x)))
        .collect();

    let output_format = config.get_output_format(&options);

    let ignore_hidden = config.get_ignore_hidden(&options);

    let mut indicator = PIndicator::build_me();
    if !config.get_disable_progress(&options) {
        indicator.spawn(output_format.clone())
    }

    let filter_modified_time = config.get_modified_time_operator(&options);
    let filter_accessed_time = config.get_accessed_time_operator(&options);
    let filter_changed_time = config.get_changed_time_operator(&options);

    let walk_data = WalkData {
        ignore_directories: ignored_full_path,
        filter_regex: &filter_regexs,
        invert_filter_regex: &invert_filter_regexs,
        allowed_filesystems,
        filter_modified_time,
        filter_accessed_time,
        filter_changed_time,
        use_apparent_size: config.get_apparent_size(&options),
        by_filecount,
        by_filetime: &by_filetime,
        ignore_hidden,
        follow_links,
        progress_data: indicator.data.clone(),
        errors: errors_for_rayon,
    };
    let threads_to_use = config.get_threads(&options);
    let stack_size = config.get_custom_stack_size(&options);
    init_rayon(&stack_size, &threads_to_use);

    let top_level_nodes = walk_it(simplified_dirs, &walk_data);

    let tree = match summarize_file_types {
        true => get_all_file_types(&top_level_nodes, number_of_lines, &by_filetime),
        false => {
            let agg_data = AggregateData {
                min_size: config.get_min_size(&options),
                only_dir: config.get_only_dir(&options),
                only_file: config.get_only_file(&options),
                number_of_lines,
                depth,
                using_a_filter: !filter_regexs.is_empty() || !invert_filter_regexs.is_empty(),
            };
            get_biggest(top_level_nodes, agg_data, &by_filetime)
        }
    };

    // Must have stopped indicator before we print to stderr
    indicator.stop();

    if errors_final.lock().unwrap().abort {
        return;
    }

    let final_errors = walk_data.errors.lock().unwrap();
    if !final_errors.file_not_found.is_empty() {
        let err = final_errors
            .file_not_found
            .iter()
            .map(|a| a.as_ref())
            .collect::<Vec<&str>>()
            .join(", ");
        eprintln!("No such file or directory: {}", err);
    }
    if !final_errors.no_permissions.is_empty() {
        if config.get_print_errors(&options) {
            let err = final_errors
                .no_permissions
                .iter()
                .map(|a| a.as_ref())
                .collect::<Vec<&str>>()
                .join(", ");
            eprintln!("Did not have permissions for directories: {}", err);
        } else {
            eprintln!(
                "Did not have permissions for all directories (add --print-errors to see errors)"
            );
        }
    }
    if !final_errors.unknown_error.is_empty() {
        let err = final_errors
            .unknown_error
            .iter()
            .map(|a| a.as_ref())
            .collect::<Vec<&str>>()
            .join(", ");
        eprintln!("Unknown Error: {}", err);
    }

    if let Some(root_node) = tree {
        let idd = InitialDisplayData {
            short_paths: !config.get_full_paths(&options),
            is_reversed: !config.get_reverse(&options),
            colors_on: is_colors,
            by_filecount,
            by_filetime,
            is_screen_reader: config.get_screen_reader(&options),
            output_format,
            bars_on_right: config.get_bars_on_right(&options),
        };

        if config.get_output_json(&options) {
            println!("{}", serde_json::to_string(&root_node).unwrap());
        } else {
            draw_it(
                idd,
                config.get_no_bars(&options),
                terminal_width,
                &root_node,
                config.get_skip_total(&options),
            )
        }
    }
}

fn init_rayon(stack_size: &Option<usize>, threads: &Option<usize>) {
    // Rayon seems to raise this error on 32-bit builds
    // The global thread pool has not been initialized.: ThreadPoolBuildError { kind: GlobalPoolAlreadyInitialized }
    if cfg!(target_pointer_width = "64") {
        let result = panic::catch_unwind(|| build_thread_pool(*stack_size, *threads));
        if result.is_err() {
            eprintln!("Problem initializing rayon, try: export RAYON_NUM_THREADS=1")
        }
    }
}

fn build_thread_pool(
    stack: Option<usize>,
    threads: Option<usize>,
) -> Result<(), rayon::ThreadPoolBuildError> {
    let mut pool = rayon::ThreadPoolBuilder::new();

    if let Some(thread_count) = threads {
        pool = pool.num_threads(thread_count);
    }

    let stack_size = match stack {
        Some(s) => Some(s),
        None => {
            let large_stack = usize::pow(1024, 3);
            let mut s = System::new();
            s.refresh_memory();
            // Larger stack size if possible to handle cases with lots of nested directories
            let available = s.available_memory();
            if available > large_stack.try_into().unwrap() {
                Some(large_stack)
            } else {
                None
            }
        }
    };
    if let Some(stack_size_param) = stack_size {
        pool = pool.stack_size(stack_size_param);
    }
    pool.build_global()
}
