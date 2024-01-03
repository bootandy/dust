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
use std::fs::read_to_string;
use std::panic;
use std::process;
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

fn init_color(no_color: bool) -> bool {
    #[cfg(windows)]
    {
        // If no color is already set do not print a warning message
        if no_color {
            true
        } else {
            // Required for windows 10
            // Fails to resolve for windows 8 so disable color
            match ansi_term::enable_ansi_support() {
                Ok(_) => no_color,
                Err(_) => {
                    eprintln!(
                    "This version of Windows does not support ANSI colors, setting no_color flag"
                );
                    true
                }
            }
        }
    }
    #[cfg(not(windows))]
    {
        no_color
    }
}

fn get_height_of_terminal() -> usize {
    // Simplify once https://github.com/eminence/terminal-size/pull/41 is
    // merged
    terminal_size()
        // Windows CI runners detect a terminal height of 0
        .map(|(_, Height(h))| max(h as usize, DEFAULT_NUMBER_OF_LINES))
        .unwrap_or(DEFAULT_NUMBER_OF_LINES)
        - 10
}

fn get_width_of_terminal() -> usize {
    // Simplify once https://github.com/eminence/terminal-size/pull/41 is
    // merged
    terminal_size()
        .map(|(Width(w), _)| match cfg!(windows) {
            // Windows CI runners detect a very low terminal width
            true => max(w as usize, DEFAULT_TERMINAL_WIDTH),
            false => w as usize,
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

    let target_dirs = match options.get_many::<String>("params") {
        Some(values) => values.map(|v| v.as_str()).collect::<Vec<&str>>(),
        None => vec!["."],
    };

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

    let no_colors = init_color(config.get_no_colors(&options));

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
    let limit_filesystem = options.get_flag("limit_filesystem");
    let follow_links = options.get_flag("dereference_links");

    let simplified_dirs = simplify_dir_names(target_dirs);
    let allowed_filesystems = limit_filesystem
        .then(|| get_filesystem_devices(simplified_dirs.iter()))
        .unwrap_or_default();

    let ignored_full_path: HashSet<PathBuf> = ignore_directories
        .into_iter()
        .flat_map(|x| simplified_dirs.iter().map(move |d| d.join(&x)))
        .collect();

    let iso = config.get_iso(&options);

    let ignore_hidden = config.get_ignore_hidden(&options);

    let mut indicator = PIndicator::build_me();
    if !config.get_disable_progress(&options) {
        indicator.spawn(iso);
    }

    let walk_data = WalkData {
        ignore_directories: ignored_full_path,
        filter_regex: &filter_regexs,
        invert_filter_regex: &invert_filter_regexs,
        allowed_filesystems,
        use_apparent_size: config.get_apparent_size(&options),
        by_filecount,
        ignore_hidden,
        follow_links,
        progress_data: indicator.data.clone(),
        errors: Arc::new(Mutex::new(RuntimeErrors::default())),
    };
    let stack_size = config.get_custom_stack_size(&options);
    init_rayon(&stack_size);

    let top_level_nodes = walk_it(simplified_dirs, &walk_data);

    let tree = match summarize_file_types {
        true => get_all_file_types(&top_level_nodes, number_of_lines),
        false => {
            let agg_data = AggregateData {
                min_size: config.get_min_size(&options, iso),
                only_dir: config.get_only_dir(&options),
                only_file: config.get_only_file(&options),
                number_of_lines,
                depth,
                using_a_filter: !filter_regexs.is_empty() || !invert_filter_regexs.is_empty(),
            };
            get_biggest(top_level_nodes, agg_data)
        }
    };

    // Must have stopped indicator before we print to stderr
    indicator.stop();

    let final_errors = walk_data.errors.lock().unwrap();
    let failed_permissions = final_errors.no_permissions;
    if !final_errors.file_not_found.is_empty() {
        let err = final_errors
            .file_not_found
            .iter()
            .map(|a| a.as_ref())
            .collect::<Vec<&str>>()
            .join(", ");
        eprintln!("No such file or directory: {}", err);
    }
    if failed_permissions {
        eprintln!("Did not have permissions for all directories");
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
            colors_on: !no_colors,
            by_filecount,
            iso,
            is_screen_reader: config.get_screen_reader(&options),
            bars_on_right: config.get_bars_on_right(&options),
        };
        draw_it(
            idd,
            config.get_no_bars(&options),
            terminal_width,
            &root_node,
            config.get_skip_total(&options),
        )
    }
}

fn init_rayon(stack_size: &Option<usize>) {
    // Rayon seems to raise this error on 32-bit builds
    // The global thread pool has not been initialized.: ThreadPoolBuildError { kind: GlobalPoolAlreadyInitialized }
    if cfg!(target_pointer_width = "64") {
        let result = panic::catch_unwind(|| {
            match stack_size {
                Some(n) => rayon::ThreadPoolBuilder::new()
                    .stack_size(*n)
                    .build_global(),
                None => {
                    let large_stack = usize::pow(1024, 3);
                    let mut s = System::new();
                    s.refresh_memory();
                    let available = s.available_memory();

                    if available > large_stack.try_into().unwrap() {
                        // Larger stack size to handle cases with lots of nested directories
                        rayon::ThreadPoolBuilder::new()
                            .stack_size(large_stack)
                            .build_global()
                    } else {
                        rayon::ThreadPoolBuilder::new().build_global()
                    }
                }
            }
        });
        if result.is_err() {
            eprintln!("Problem initializing rayon, try: export RAYON_NUM_THREADS=1")
        }
    }
}
