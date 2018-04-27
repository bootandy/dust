#[macro_use]
extern crate clap;
extern crate assert_cli;
extern crate walkdir;

use self::display::draw_it;
use clap::{App, AppSettings, Arg};
use std::io::{self, Write};
use utils::{find_big_ones, get_dir_tree, sort};

mod display;
mod utils;

static DEFAULT_NUMBER_OF_LINES: &'static str = "15";

fn main() {
    let options = App::new("Dust")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("depth")
                .short("d")
                .long("depth")
                .help("Depth to show")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("number_of_lines")
                .short("n")
                .long("number-of-lines")
                .help("Number of lines of output to show")
                .takes_value(true)
                .default_value(DEFAULT_NUMBER_OF_LINES),
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
        .arg(Arg::with_name("inputs").multiple(true))
        .get_matches();

    let filenames = {
        match options.values_of("inputs") {
            None => vec!["."],
            Some(r) => r.collect(),
        }
    };

    let number_of_lines = value_t!(options.value_of("number_of_lines"), usize).unwrap();
    let depth = {
        if options.is_present("depth") {
            match value_t!(options.value_of("depth"), u64) {
                Ok(v) => Some(v + 1),
                Err(_) => None,
            }
        } else {
            None
        }
    };
    if options.is_present("depth")
        && options.value_of("number_of_lines").unwrap() != DEFAULT_NUMBER_OF_LINES
    {
        io::stderr()
            .write(b"Use either -n for number of directories to show. Or -d for depth. Not both")
            .expect("Error writing to stderr. Oh the irony!");
        return;
    }

    let use_apparent_size = options.is_present("display_apparent_size");
    let use_full_path = options.is_present("display_full_paths");

    let (permissions, nodes, top_level_names) = get_dir_tree(&filenames, use_apparent_size);
    let sorted_data = sort(nodes);
    let biggest_ones = {
        if depth.is_none() {
            find_big_ones(sorted_data, number_of_lines)
        } else {
            sorted_data
        }
    };
    draw_it(
        permissions,
        !use_full_path,
        depth,
        top_level_names,
        biggest_ones,
    );
}

#[cfg(test)]
mod tests;
