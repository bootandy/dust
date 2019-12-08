#[macro_use]
extern crate clap;

use self::display::draw_it;
use crate::utils::is_a_parent_of;
use clap::{App, AppSettings, Arg};
use utils::{find_big_ones, get_dir_tree, simplify_dir_names, sort, trim_deep_ones, Node};

mod display;
mod utils;

static DEFAULT_NUMBER_OF_LINES: usize = 20;

fn main() {
    let def_num_str = DEFAULT_NUMBER_OF_LINES.to_string();
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
            Arg::with_name("display_apparent_size")
                .short("s")
                .long("apparent-size")
                .help("If set will use file length. Otherwise we use blocks"),
        )
        .arg(
            Arg::with_name("reverse")
                .short("r")
                .long("reverse")
                .help("If applied tree will be printed upside down (biggest lowest)"),
        )
        .arg(
            Arg::with_name("no_colors")
                .short("c")
                .long("no_colors")
                .help("If applied no colors will be printed (normally largest directories are marked in red"),
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
            DEFAULT_NUMBER_OF_LINES
        }
    };

    let threads = options.value_of("threads").and_then(|threads| {
        threads
            .parse::<usize>()
            .map_err(|_| eprintln!("Ignoring bad value for threads: {:?}", threads))
            .ok()
    });

    let depth = options.value_of("depth").and_then(|depth| {
        depth
            .parse::<u64>()
            .map(|v| v + 1)
            .map_err(|_| eprintln!("Ignoring bad value for depth"))
            .ok()
    });
    if options.is_present("depth") && number_of_lines != DEFAULT_NUMBER_OF_LINES {
        eprintln!("Use either -n or -d. Not both");
        return;
    }

    let use_apparent_size = options.is_present("display_apparent_size");
    let use_full_path = options.is_present("display_full_paths");

    let simplified_dirs = simplify_dir_names(target_dirs);
    let (permissions, nodes) = get_dir_tree(&simplified_dirs, use_apparent_size, threads);
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
        use_full_path,
        options.is_present("reverse"),
        !options.is_present("no_colors"),
        tree,
    );
}

fn build_tree(biggest_ones: Vec<(String, u64)>, depth: Option<u64>) -> Node {
    let mut top_parent = Node::default();

    // assume sorted order
    for b in biggest_ones {
        let n = Node {
            name: b.0,
            size: b.1,
            children: Vec::default(),
        };
        recursively_build_tree(&mut top_parent, n, depth);
        top_parent.children.sort_unstable()
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

#[cfg(test)]
mod tests;
