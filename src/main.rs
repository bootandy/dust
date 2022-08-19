extern crate clap;
extern crate rayon;
extern crate regex;
extern crate unicode_width;

use std::collections::HashSet;
use std::process;

use self::display::draw_it;
use clap::{crate_version, Arg};
use clap::{Command, Values};
use config::get_config;
use dir_walker::{walk_it, WalkData};
use filter::{get_all_file_types, get_biggest};
use regex::Regex;
use std::cmp::max;
use std::path::PathBuf;
use terminal_size::{terminal_size, Height, Width};
use utils::get_filesystem_devices;
use utils::simplify_dir_names;

mod config;
mod dir_walker;
mod display;
mod display_node;
mod filter;
mod node;
mod platform;
mod utils;

static DEFAULT_NUMBER_OF_LINES: usize = 30;
static DEFAULT_TERMINAL_WIDTH: usize = 80;

#[cfg(windows)]
fn init_color(no_color: bool) -> bool {
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
fn init_color(no_color: bool) -> bool {
    no_color
}

fn get_height_of_terminal() -> usize {
    // Windows CI runners detect a terminal height of 0
    if let Some((Width(_w), Height(h))) = terminal_size() {
        max(h as usize, DEFAULT_NUMBER_OF_LINES) - 10
    } else {
        DEFAULT_NUMBER_OF_LINES - 10
    }
}

#[cfg(windows)]
fn get_width_of_terminal() -> usize {
    // Windows CI runners detect a very low terminal width
    if let Some((Width(w), Height(_h))) = terminal_size() {
        max(w as usize, DEFAULT_TERMINAL_WIDTH)
    } else {
        DEFAULT_TERMINAL_WIDTH
    }
}

#[cfg(not(windows))]
fn get_width_of_terminal() -> usize {
    if let Some((Width(w), Height(_h))) = terminal_size() {
        w as usize
    } else {
        DEFAULT_TERMINAL_WIDTH
    }
}

fn get_regex_value(maybe_value: Option<Values>) -> Vec<Regex> {
    let mut result = vec![];
    if let Some(v) = maybe_value {
        for reg in v {
            match Regex::new(reg) {
                Ok(r) => result.push(r),
                Err(e) => {
                    eprintln!("Ignoring bad value for regex {:?}", e);
                    process::exit(1);
                }
            }
        }
    }
    result
}

fn main() {
    let options = Command::new("Dust")
        .about("Like du but more intuitive")
        .version(crate_version!())
        .trailing_var_arg(true)
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .help("Depth to show")
                .takes_value(true)
                .default_value(usize::MAX.to_string().as_ref())
        )
        .arg(
            Arg::new("number_of_lines")
                .short('n')
                .long("number-of-lines")
                .help("Number of lines of output to show. (Default is terminal_height - 10)")
                .takes_value(true)
        )
        .arg(
            Arg::new("display_full_paths")
                .short('p')
                .long("full-paths")
                .help("Subdirectories will not have their path shortened"),
        )
        .arg(
            Arg::new("ignore_directory")
                .short('X')
                .long("ignore-directory")
                .takes_value(true)
                .number_of_values(1)
                .multiple_occurrences(true)
                .help("Exclude any file or directory with this name"),
        )
        .arg(
            Arg::new("limit_filesystem")
                .short('x')
                .long("limit-filesystem")
                .help("Only count the files and directories on the same filesystem as the supplied directory"),
        )
        .arg(
            Arg::new("display_apparent_size")
                .short('s')
                .long("apparent-size")
                .help("Use file length instead of blocks"),
        )
        .arg(
            Arg::new("reverse")
                .short('r')
                .long("reverse")
                .help("Print tree upside down (biggest highest)"),
        )
        .arg(
            Arg::new("no_colors")
                .short('c')
                .long("no-colors")
                .help("No colors will be printed (Useful for commands like: watch)"),
        )
        .arg(
            Arg::new("no_bars")
                .short('b')
                .long("no-percent-bars")
                .help("No percent bars or percentages will be displayed"),
        )
        .arg(
            Arg::new("skip_total")
                .long("skip-total")
                .help("No total row will be displayed"),
        )
        .arg(
            Arg::new("by_filecount")
                .short('f')
                .long("filecount")
                .help("Directory 'size' is number of child files/dirs not disk size"),
        )
        .arg(
            Arg::new("ignore_hidden")
                .short('i') // Do not use 'h' this is used by 'help'
                .long("ignore_hidden")
                .help("Do not display hidden files"),
        )
        .arg(
            Arg::new("invert_filter")
                .short('v')
                .long("invert-filter")
                .takes_value(true)
                .number_of_values(1)
                .multiple_occurrences(true)
                .conflicts_with("filter")
                .conflicts_with("types")
                .help("Exclude filepaths matching this regex. To ignore png files type: -v \"\\.png$\" "),
        )
        .arg(
            Arg::new("filter")
                .short('e')
                .long("filter")
                .takes_value(true)
                .number_of_values(1)
                .multiple_occurrences(true)
                .conflicts_with("types")
                .help("Only include filepaths matching this regex. For png files type: -e \"\\.png$\" "),
        )
        .arg(
            Arg::new("types")
                .short('t')
                .long("file_types")
                .conflicts_with("depth")
                .help("show only these file types"),
        )
        .arg(
            Arg::new("width")
                .short('w')
                .long("terminal_width")
                .takes_value(true)
                .number_of_values(1)
                .help("Specify width of output overriding the auto detection of terminal width"),
        )
        .arg(
            Arg::new("iso")
                .short('H')
                .long("si")
                .help("print sizes in powers of 1000 (e.g., 1.1G)")
        )
        .arg(Arg::new("inputs").multiple_occurrences(true).default_value("."))
        .get_matches();

    let config = get_config();

    let target_dirs = options
        .values_of("inputs")
        .expect("Should be a default value here")
        .collect();

    let summarize_file_types = options.is_present("types");

    let filter_regexs = get_regex_value(options.values_of("filter"));
    let invert_filter_regexs = get_regex_value(options.values_of("invert_filter"));

    let terminal_width = match options.value_of_t("width") {
        Ok(v) => v,
        Err(_) => get_width_of_terminal(),
    };

    let depth = match options.value_of_t("depth") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Ignoring bad value for depth");
            usize::MAX
        }
    };
    // If depth is set we set the default number_of_lines to be max
    // instead of screen height
    let default_height = if depth != usize::MAX {
        usize::MAX
    } else {
        get_height_of_terminal()
    };

    let number_of_lines = match options.value_of("number_of_lines") {
        Some(v) => match v.parse::<usize>() {
            Ok(num_lines) => num_lines,
            Err(_) => {
                eprintln!("Ignoring bad value for number_of_lines");
                default_height
            }
        },
        None => default_height,
    };

    let no_colors = init_color(config.get_no_colors(&options));

    let ignore_directories: Vec<PathBuf> = options
        .values_of("ignore_directory")
        .map(|i| i.map(PathBuf::from).collect())
        .unwrap_or_default();

    let by_filecount = options.is_present("by_filecount");
    let limit_filesystem = options.is_present("limit_filesystem");

    let simplified_dirs = simplify_dir_names(target_dirs);
    let allowed_filesystems = {
        if limit_filesystem {
            get_filesystem_devices(simplified_dirs.iter())
        } else {
            HashSet::new()
        }
    };

    let ignored_full_path: HashSet<PathBuf> = ignore_directories
        .into_iter()
        .flat_map(|x| simplified_dirs.iter().map(move |d| d.join(x.clone())))
        .collect();

    let walk_data = WalkData {
        ignore_directories: ignored_full_path,
        filter_regex: &filter_regexs,
        invert_filter_regex: &invert_filter_regexs,
        allowed_filesystems,
        use_apparent_size: config.get_apparent_size(&options),
        by_filecount,
        ignore_hidden: config.get_ignore_hidden(&options),
    };
    // Larger stack size to handle cases with lots of nested directories
    rayon::ThreadPoolBuilder::new()
        .stack_size(usize::pow(1024, 3))
        .build_global()
        .unwrap();

    let (top_level_nodes, has_errors) = walk_it(simplified_dirs, walk_data);

    let tree = {
        match (depth, summarize_file_types) {
            (_, true) => get_all_file_types(top_level_nodes, number_of_lines),
            (depth, _) => get_biggest(
                top_level_nodes,
                number_of_lines,
                depth,
                options.values_of("filter").is_some()
                    || options.value_of("invert_filter").is_some(),
            ),
        }
    };

    if has_errors {
        eprintln!("Did not have permissions for all directories");
    }
    match tree {
        None => {}
        Some(root_node) => draw_it(
            config.get_full_paths(&options),
            !config.get_reverse(&options),
            no_colors,
            config.get_no_bars(&options),
            terminal_width,
            by_filecount,
            root_node,
            config.get_iso(&options),
            config.get_skip_total(&options),
        ),
    }
}
