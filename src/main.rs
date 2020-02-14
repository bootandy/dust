#[macro_use]
extern crate clap;
extern crate unicode_width;

use self::display::draw_it;
use crate::utils::is_a_parent_of;
use clap::{App, AppSettings, Arg};
use std::path::PathBuf;
use terminal_size::{terminal_size, Height, Width};
use utils::{find_big_ones, get_dir_tree, simplify_dir_names, sort, trim_deep_ones, Node};

mod display;
mod utils;

static DEFAULT_NUMBER_OF_LINES: usize = 20;

#[cfg(windows)]
fn init_color() {
    ansi_term::enable_ansi_support().expect("Couldn't enable color support");
}

#[cfg(not(windows))]
fn init_color() {}

fn get_height_of_terminal() -> usize {
    // Windows CI runners detect a terminal height of 0
    let default_height = {
        if let Some((Width(_w), Height(h))) = terminal_size() {
            h as usize
        } else {
            0
        }
    };
    if default_height < DEFAULT_NUMBER_OF_LINES {
        DEFAULT_NUMBER_OF_LINES
    } else {
        default_height - 10
    }
}

fn main() {
    init_color();

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
                .takes_value(true),
        )
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .help("Number of threads to spawn simultaneously")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("number_of_lines")
                .short("n")
                .long("number-of-lines")
                .help("Number of lines of output to show")
                .takes_value(true)
                .default_value(def_num_str.as_ref()),
        )
        .arg(
            Arg::with_name("display_full_paths")
                .short("p")
                .long("full-paths")
                .help("If set sub directories will not have their path shortened"),
        )
        .arg(
            Arg::with_name("ignore_directory")
                .short("X")
                .long("ignore-directory")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .help("Exclude any file or directory with this name."),
        )
        .arg(
            Arg::with_name("limit_filesystem")
                .short("x")
                .long("limit-filesystem")
                .help("Only count the files and directories in the same filesystem as the supplied directory"),
        )
        .arg(
            Arg::with_name("display_apparent_size")
                .short("s")
                .long("apparent-size")
                .help("If set will use file length. Otherwise we use blocks"),
        )
        .arg(
            Arg::with_name("reverse")
                .short("r")
                .long("reverse")
                .help("If applied tree will be printed upside down (biggest highest)"),
        )
        .arg(
            Arg::with_name("no_colors")
                .short("c")
                .long("no_colors")
                .help("If applied no colors will be printed (normally largest directories are marked in red"),
        )
        .arg(
            Arg::with_name("no_bars")
                .short("b")
                .long("no_percent_bars")
                .help("If applied no percent bars or percents will be displayed"),
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

    let temp_threads = options.value_of("threads").and_then(|threads| {
        threads
            .parse::<usize>()
            .map_err(|_| eprintln!("Ignoring bad value for threads: {:?}", threads))
            .ok()
    });
    // Bug in JWalk
    // https://github.com/jessegrosjean/jwalk/issues/15
    // We force it to use 2 threads if there is only 1 cpu
    // as JWalk breaks if it tries to run on a single cpu
    let threads = {
        if temp_threads.is_none() && num_cpus::get() == 1 {
            Some(2)
        } else {
            temp_threads
        }
    };

    let depth = options.value_of("depth").and_then(|depth| {
        depth
            .parse::<u64>()
            .map(|v| v + 1)
            .map_err(|_| eprintln!("Ignoring bad value for depth"))
            .ok()
    });
    if options.is_present("depth") && number_of_lines != default_height {
        eprintln!("Use either -n or -d. Not both");
        return;
    }

    let use_apparent_size = options.is_present("display_apparent_size");
    let limit_filesystem = options.is_present("limit_filesystem");
    let ignore_directories = match options.values_of("ignore_directory") {
        Some(i) => Some(i.map(PathBuf::from).collect()),
        None => None,
    };

    let simplified_dirs = simplify_dir_names(target_dirs);
    let (permissions, nodes) = get_dir_tree(
        &simplified_dirs,
        &ignore_directories,
        use_apparent_size,
        limit_filesystem,
        threads,
    );
    let sorted_data = sort(nodes);
    let biggest_ones = {
        match depth {
            None => find_big_ones(sorted_data, number_of_lines + simplified_dirs.len()),
            Some(d) => trim_deep_ones(sorted_data, d, &simplified_dirs),
        }
    };
    let tree = build_tree(biggest_ones, depth);

    draw_it(
        permissions,
        options.is_present("display_full_paths"),
        !options.is_present("reverse"),
        options.is_present("no_colors"),
        options.is_present("no_bars"),
        tree,
    );
}

fn build_tree(biggest_ones: Vec<(PathBuf, u64)>, depth: Option<u64>) -> Node {
    let mut top_parent = Node::default();

    // assume sorted order
    for b in biggest_ones {
        let n = Node {
            name: b.0,
            size: b.1,
            children: Vec::default(),
        };
        recursively_build_tree(&mut top_parent, n, depth);
    }
    top_parent
}

fn recursively_build_tree(parent_node: &mut Node, new_node: Node, depth: Option<u64>) {
    let new_depth = match depth {
        None => None,
        Some(0) => return,
        Some(d) => Some(d - 1),
    };
    if let Some(c) = parent_node
        .children
        .iter_mut()
        .find(|c| is_a_parent_of(&c.name, &new_node.name))
    {
        recursively_build_tree(c, new_node, new_depth);
    } else {
        parent_node.children.push(new_node);
    }
}
