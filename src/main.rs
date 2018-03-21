extern crate dust;
use dust::*;

extern crate ansi_term;
use ansi_term::Colour::Fixed;
#[macro_use]
extern crate clap;
use clap::{App, AppSettings, Arg};

static DEFAULT_NUMBER_OF_LINES: &'static str = "15";

fn main() {
    let options = App::new("Trailing args example")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("number_of_lines")
                .short("n")
                .help("Number of lines of output to show")
                .takes_value(true)
                .default_value(DEFAULT_NUMBER_OF_LINES),
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

    let (permissions, results) = get_dir_tree(&filenames);
    let slice_it = find_big_ones(&results, number_of_lines);
    display(permissions, &slice_it);
}
