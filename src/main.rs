mod cli;
mod config;
mod dir_walker;
mod display;
mod display_node;
mod filter;
mod filter_type;
mod node;
mod platform;
mod utils;

use crate::cli::build_cli;
use dir_walker::WalkData;
use filter::AggregateData;
use std::collections::HashSet;
use std::io::BufRead;
use std::process;
use sysinfo::{System, SystemExt};

use self::display::draw_it;
use clap::Values;
use config::get_config;
use dir_walker::walk_it;
use filter::get_biggest;
use filter_type::get_all_file_types;
use rayon::ThreadPoolBuildError;
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

fn get_regex_value(maybe_value: Option<Values>) -> Vec<Regex> {
    maybe_value
        .unwrap_or_default()
        .map(|reg| {
            Regex::new(reg).unwrap_or_else(|err| {
                eprintln!("Ignoring bad value for regex {:?}", err);
                process::exit(1)
            })
        })
        .collect()
}

// Returns a list of lines from stdin or `None` if there's nothing to read
fn get_lines_from_stdin() -> Option<Vec<String>> {
    atty::isnt(atty::Stream::Stdin).then(|| {
        std::io::stdin()
            .lock()
            .lines()
            .collect::<Result<_, _>>()
            .expect("Error reading from stdin")
    })
}

fn main() {
    let options = build_cli().get_matches();
    let config = get_config();
    let stdin_lines = get_lines_from_stdin();

    let target_dirs = match options.values_of("inputs") {
        Some(values) => values.collect(),
        None => stdin_lines.as_ref().map_or(vec!["."], |lines| {
            lines.iter().map(String::as_str).collect()
        }),
    };

    let summarize_file_types = options.is_present("types");

    let filter_regexs = get_regex_value(options.values_of("filter"));
    let invert_filter_regexs = get_regex_value(options.values_of("invert_filter"));

    let terminal_width = options
        .value_of_t("width")
        .unwrap_or_else(|_| get_width_of_terminal());

    let depth = options.value_of_t("depth").unwrap_or(usize::MAX);

    // If depth is set, then we set the default number_of_lines to be max
    // instead of screen height
    let default_height = if depth != usize::MAX {
        usize::MAX
    } else {
        get_height_of_terminal()
    };

    let number_of_lines = options
        .value_of("number_of_lines")
        .and_then(|v| {
            v.parse()
                .map_err(|_| eprintln!("Ignoring bad value for number_of_lines"))
                .ok()
        })
        .unwrap_or(default_height);

    let no_colors = init_color(config.get_no_colors(&options));

    let ignore_directories = options
        .values_of("ignore_directory")
        .unwrap_or_default()
        .map(PathBuf::from);

    let by_filecount = options.is_present("by_filecount");
    let limit_filesystem = options.is_present("limit_filesystem");
    let follow_links = options.is_present("dereference_links");

    let simplified_dirs = simplify_dir_names(target_dirs);
    let allowed_filesystems = limit_filesystem
        .then(|| get_filesystem_devices(simplified_dirs.iter()))
        .unwrap_or_default();

    let ignored_full_path: HashSet<PathBuf> = ignore_directories
        .flat_map(|x| simplified_dirs.iter().map(move |d| d.join(&x)))
        .collect();

    let walk_data = WalkData {
        ignore_directories: ignored_full_path,
        filter_regex: &filter_regexs,
        invert_filter_regex: &invert_filter_regexs,
        allowed_filesystems,
        use_apparent_size: config.get_apparent_size(&options),
        by_filecount,
        ignore_hidden: config.get_ignore_hidden(&options),
        follow_links,
    };
    let _rayon = init_rayon();

    let iso = config.get_iso(&options);
    let (top_level_nodes, has_errors) = walk_it(simplified_dirs, walk_data);
    let tree = match summarize_file_types {
        true => get_all_file_types(&top_level_nodes, number_of_lines),
        false => {
            let agg_data = AggregateData {
                min_size: config.get_min_size(&options, iso),
                only_dir: config.get_only_dir(&options),
                number_of_lines,
                depth,
                using_a_filter: options.values_of("filter").is_some()
                    || options.value_of("invert_filter").is_some(),
            };
            get_biggest(top_level_nodes, agg_data)
        }
    };

    if has_errors {
        eprintln!("Did not have permissions for all directories");
    }
    if let Some(root_node) = tree {
        draw_it(
            config.get_full_paths(&options),
            !config.get_reverse(&options),
            no_colors,
            config.get_no_bars(&options),
            terminal_width,
            by_filecount,
            &root_node,
            iso,
            config.get_skip_total(&options),
        )
    }
}

fn init_rayon() -> Result<(), ThreadPoolBuildError> {
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
        Ok(())
    }
}
