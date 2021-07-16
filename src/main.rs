#[macro_use]
extern crate clap;
extern crate rayon;
extern crate unicode_width;

use std::collections::HashSet;

use self::display::draw_it;
use clap::{App, AppSettings, Arg};
use dir_walker::walk_it;
use dir_walker::WalkData;
use filter::{get_biggest, get_by_depth};
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

fn get_width_of_terminal() -> usize {
    // Windows CI runners detect a very low terminal width
    if let Some((Width(w), Height(_h))) = terminal_size() {
        max(w as usize, DEFAULT_TERMINAL_WIDTH)
    } else {
        DEFAULT_TERMINAL_WIDTH
    }
}

fn main() {
    let default_height = get_height_of_terminal();
    let def_num_str = default_height.to_string();

    let options = App::new("Dust")
        .about("Like du but more intuitive")
        .version(crate_version!())
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("depth")
                .short("d")
                .long("depth")
                .help("Depth to show")
                .takes_value(true)
                .conflicts_with("number_of_lines"),
        )
        .arg(
            Arg::with_name("number_of_lines")
                .short("n")
                .long("number-of-lines")
                .help("Number of lines of output to show. This is Height, (but h is help)")
                .takes_value(true)
                .default_value(def_num_str.as_ref()),
        )
        .arg(
            Arg::with_name("display_full_paths")
                .short("p")
                .long("full-paths")
                .help("Subdirectories will not have their path shortened"),
        )
        .arg(
            Arg::with_name("ignore_directory")
                .short("X")
                .long("ignore-directory")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .help("Exclude any file or directory with this name"),
        )
        .arg(
            Arg::with_name("limit_filesystem")
                .short("x")
                .long("limit-filesystem")
                .help("Only count the files and directories on the same filesystem as the supplied directory"),
        )
        .arg(
            Arg::with_name("display_apparent_size")
                .short("s")
                .long("apparent-size")
                .help("Use file length instead of blocks"),
        )
        .arg(
            Arg::with_name("reverse")
                .short("r")
                .long("reverse")
                .help("Print tree upside down (biggest highest)"),
        )
        .arg(
            Arg::with_name("no_colors")
                .short("c")
                .long("no-colors")
                .help("No colors will be printed (normally largest directories are colored)"),
        )
        .arg(
            Arg::with_name("no_bars")
                .short("b")
                .long("no-percent-bars")
                .help("No percent bars or percentages will be displayed"),
        )
        .arg(
            Arg::with_name("by_filecount")
                .short("f")
                .long("filecount")
                .help("Directory 'size' is number of child files/dirs not disk size"),
        )
        .arg(
            Arg::with_name("ignore_hidden")
                .short("i") // Do not use 'h' this is used by 'help'
                .long("ignore_hidden")
                .help("Do not display hidden files"),
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("terminal_width")
                .takes_value(true)
                .number_of_values(1)
                .help("Specify width of output overriding the auto detection of terminal width"),
        )
        .arg(Arg::with_name("inputs").multiple(true))
        .get_matches();

    let target_dirs = {
        match options.values_of("inputs") {
            None => vec!["."],
            Some(r) => r.collect(),
        }
    };

    let number_of_lines = match value_t!(options.value_of("number_of_lines"), usize) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Ignoring bad value for number_of_lines");
            default_height
        }
    };

    let terminal_width = match value_t!(options.value_of("width"), usize) {
        Ok(v) => v,
        Err(_) => get_width_of_terminal(),
    };

    let depth = options.value_of("depth").and_then(|depth| {
        depth
            .parse::<usize>()
            .map(|v| v + 1)
            .map_err(|_| eprintln!("Ignoring bad value for depth"))
            .ok()
    });

    let no_colors = init_color(options.is_present("no_colors"));
    let use_apparent_size = options.is_present("display_apparent_size");
    let ignore_directories: Vec<PathBuf> = options
        .values_of("ignore_directory")
        .map(|i| i.map(PathBuf::from).collect())
        .unwrap_or_default();

    let by_filecount = options.is_present("by_filecount");
    let ignore_hidden = options.is_present("ignore_hidden");
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
        allowed_filesystems,
        use_apparent_size,
        by_filecount,
        ignore_hidden,
    };

    let (nodes, errors) = walk_it(simplified_dirs, walk_data);

    let tree = {
        match depth {
            None => get_biggest(nodes, number_of_lines),
            Some(depth) => get_by_depth(nodes, depth),
        }
    };

    draw_it(
        errors,
        options.is_present("display_full_paths"),
        !options.is_present("reverse"),
        no_colors,
        options.is_present("no_bars"),
        terminal_width,
        by_filecount,
        tree,
    );
}
