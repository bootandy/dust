extern crate clap;
extern crate rayon;
extern crate regex;
extern crate unicode_width;

use std::collections::HashSet;
use std::process;

use self::display::draw_it;
use clap::{crate_version, Arg};
use clap::{Command, Values};
use dir_walker::{walk_it, WalkData};
use filter::{get_all_file_types, get_biggest};
use regex::Regex;
use std::cmp::max;
use std::path::PathBuf;
use terminal_size::{terminal_size, Height, Width};
use utils::get_filesystem_devices;
use utils::simplify_dir_names;

mod dir_walker;
mod display;
mod display_node;
mod filter;
mod node;
mod platform;
mod utils;

static DEFAULT_NUMBER_OF_LINES: usize = 30;
static DEFAULT_TERMINAL_WIDTH: usize = 80;

#[cfg(not(windows))]
fn init_color(no_color: bool) -> bool {
    no_color
}

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
        .arg(Arg::new("inputs").multiple_occurrences(true).default_value("."))
        .arg(
            Arg::new("iso")
                .short('H')
                .long("si")
                .help("print sizes in powers of 1000 (e.g., 1.1G)")
        )
        .get_matches();

    let target_dirs = options
        .values_of("inputs")
        .expect("Should be a default value here")
        .collect();

    let summarize_file_types = options.is_present("types");

    let filter_regexs = get_regex_value(options.values_of("filter"));
    let invert_filter_regexs = get_regex_value(options.values_of("invert_filter"));

    let terminal_width = options
        .value_of_t("width")
        .unwrap_or_else(|_| get_width_of_terminal());

    let depth = options.value_of_t("depth").unwrap_or_else(|_| {
        eprintln!("Ignoring bad value for depth");
        usize::MAX
    });

    // If depth is set we set the default number_of_lines to be max
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

    let no_colors = init_color(options.is_present("no_colors"));
    let use_apparent_size = options.is_present("display_apparent_size");
    let ignore_directories = options
        .values_of("ignore_directory")
        .unwrap_or_default()
        .map(PathBuf::from);

    let by_filecount = options.is_present("by_filecount");
    let ignore_hidden = options.is_present("ignore_hidden");
    let limit_filesystem = options.is_present("limit_filesystem");

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
        use_apparent_size,
        by_filecount,
        ignore_hidden,
    };
    // Larger stack size to handle cases with lots of nested directories
    rayon::ThreadPoolBuilder::new()
        .stack_size(usize::pow(1024, 3))
        .build_global()
        .unwrap();

    let (top_level_nodes, has_errors) = walk_it(simplified_dirs, walk_data);

    let tree = match summarize_file_types {
        true => get_all_file_types(&top_level_nodes, number_of_lines),
        false => get_biggest(
            top_level_nodes,
            number_of_lines,
            depth,
            options.values_of("filter").is_some() || options.value_of("invert_filter").is_some(),
        ),
    };

    if has_errors {
        eprintln!("Did not have permissions for all directories");
    }

    if let Some(root_node) = tree {
        draw_it(
            options.is_present("display_full_paths"),
            !options.is_present("reverse"),
            no_colors,
            options.is_present("no_bars"),
            terminal_width,
            by_filecount,
            &root_node,
            options.is_present("iso"),
            options.is_present("skip_total"),
        )
    }
}
