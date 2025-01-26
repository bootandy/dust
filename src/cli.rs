use clap::{builder::PossibleValue, value_parser, Arg, Command};

// For single thread mode set this variable on your command line:
// export RAYON_NUM_THREADS=1

pub fn build_cli() -> Command {
    Command::new("Dust")
        .about("Like du but more intuitive")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .value_name("DEPTH")
                .value_parser(value_parser!(usize))
                .help("Depth to show")
                .num_args(1)
        )
        .arg(
            Arg::new("threads")
                .short('T')
                .long("threads")
                .value_parser(value_parser!(usize))
                .help("Number of threads to use")
                .num_args(1)
        )
        .arg(
            Arg::new("config")
                .long("config")
                .help("Specify a config file to use")
                .value_name("FILE")
                .value_hint(clap::ValueHint::FilePath)
                .value_parser(value_parser!(String))
                .num_args(1)
        )
        .arg(
            Arg::new("number_of_lines")
                .short('n')
                .long("number-of-lines")
                .value_name("NUMBER")
                .value_parser(value_parser!(usize))
                .help("Number of lines of output to show. (Default is terminal_height - 10)")
                .num_args(1)
        )
        .arg(
            Arg::new("display_full_paths")
                .short('p')
                .long("full-paths")
                .action(clap::ArgAction::SetTrue)
                .help("Subdirectories will not have their path shortened"),
        )
        .arg(
            Arg::new("ignore_directory")
                .short('X')
                .long("ignore-directory")
                .value_name("PATH")
                .value_hint(clap::ValueHint::AnyPath)
                .action(clap::ArgAction::Append)
                .help("Exclude any file or directory with this path"),
        )
        .arg(
            Arg::new("ignore_all_in_file")
                .short('I')
                .long("ignore-all-in-file")
                .value_name("FILE")
                .value_hint(clap::ValueHint::FilePath)
                .value_parser(value_parser!(String))
                .help("Exclude any file or directory with a regex matching that listed in this file, the file entries will be added to the ignore regexs provided by --invert_filter"),
        )
         .arg(
            Arg::new("dereference_links")
                .short('L')
                .long("dereference-links")
                .action(clap::ArgAction::SetTrue)
                .help("dereference sym links - Treat sym links as directories and go into them"),
        )
        .arg(
            Arg::new("limit_filesystem")
                .short('x')
                .long("limit-filesystem")
                .action(clap::ArgAction::SetTrue)
                .help("Only count the files and directories on the same filesystem as the supplied directory"),
        )
        .arg(
            Arg::new("display_apparent_size")
                .short('s')
                .long("apparent-size")
                .action(clap::ArgAction::SetTrue)
                .help("Use file length instead of blocks"),
        )
        .arg(
            Arg::new("reverse")
                .short('r')
                .long("reverse")
                .action(clap::ArgAction::SetTrue)
                .help("Print tree upside down (biggest highest)"),
        )
        .arg(
            Arg::new("no_colors")
                .short('c')
                .long("no-colors")
                .action(clap::ArgAction::SetTrue)
                .help("No colors will be printed (Useful for commands like: watch)"),
        )
        .arg(
            Arg::new("force_colors")
                .short('C')
                .long("force-colors")
                .action(clap::ArgAction::SetTrue)
                .help("Force colors print"),
        )
        .arg(
            Arg::new("no_bars")
                .short('b')
                .long("no-percent-bars")
                .action(clap::ArgAction::SetTrue)
                .help("No percent bars or percentages will be displayed"),
        )
        .arg(
            Arg::new("bars_on_right")
                .short('B')
                .long("bars-on-right")
                .action(clap::ArgAction::SetTrue)
                .help("percent bars moved to right side of screen"),
        )
        .arg(
            Arg::new("min_size")
                .short('z')
                .long("min-size")
                .value_name("MIN_SIZE")
                .num_args(1)
                .help("Minimum size file to include in output"),
        )
        .arg(
            Arg::new("screen_reader")
                .short('R')
                .long("screen-reader")
                .action(clap::ArgAction::SetTrue)
                .help("For screen readers. Removes bars. Adds new column: depth level (May want to use -p too for full path)"),
        )
        .arg(
            Arg::new("skip_total")
                .long("skip-total")
                .action(clap::ArgAction::SetTrue)
                .help("No total row will be displayed"),
        )
        .arg(
            Arg::new("by_filecount")
                .short('f')
                .long("filecount")
                .action(clap::ArgAction::SetTrue)
                .help("Directory 'size' is number of child files instead of disk size"),
        )
        .arg(
            Arg::new("ignore_hidden")
                .short('i') // Do not use 'h' this is used by 'help'
                .long("ignore_hidden")
                .action(clap::ArgAction::SetTrue)
                .help("Do not display hidden files"),
        )
        .arg(
            Arg::new("invert_filter")
                .short('v')
                .long("invert-filter")
                .value_name("REGEX")
                .action(clap::ArgAction::Append)
                .conflicts_with("filter")
                .conflicts_with("types")
                .help("Exclude filepaths matching this regex. To ignore png files type: -v \"\\.png$\" "),
        )
        .arg(
            Arg::new("filter")
                .short('e')
                .long("filter")
                .value_name("REGEX")
                .action(clap::ArgAction::Append)
                .conflicts_with("types")
                .help("Only include filepaths matching this regex. For png files type: -e \"\\.png$\" "),
        )
        .arg(
            Arg::new("types")
                .short('t')
                .long("file_types")
                .conflicts_with("depth")
                .conflicts_with("only_dir")
                .action(clap::ArgAction::SetTrue)
                .help("show only these file types"),
        )
        .arg(
            Arg::new("width")
                .short('w')
                .long("terminal_width")
                .value_name("WIDTH")
                .value_parser(value_parser!(usize))
                .num_args(1)
                .help("Specify width of output overriding the auto detection of terminal width"),
        )
        .arg(
            Arg::new("disable_progress")
                .short('P')
                .long("no-progress")
                .action(clap::ArgAction::SetTrue)
                .help("Disable the progress indication."),
        )
        .arg(
            Arg::new("print_errors")
                .long("print-errors")
                .action(clap::ArgAction::SetTrue)
                .help("Print path with errors."),
        )
        .arg(
            Arg::new("only_dir")
                .short('D')
                .long("only-dir")
                .conflicts_with("only_file")
                .conflicts_with("types")
                .action(clap::ArgAction::SetTrue)
                .help("Only directories will be displayed."),
        )
        .arg(
            Arg::new("only_file")
                .short('F')
                .long("only-file")
                .conflicts_with("only_dir")
                .action(clap::ArgAction::SetTrue)
                .help("Only files will be displayed. (Finds your largest files)"),
        )
        .arg(
            Arg::new("output_format")
                .short('o')
                .long("output-format")
                .value_name("FORMAT")
                .value_parser([
                    PossibleValue::new("si"),
                    PossibleValue::new("b"),
                    PossibleValue::new("k").alias("kib"),
                    PossibleValue::new("m").alias("mib"),
                    PossibleValue::new("g").alias("gib"),
                    PossibleValue::new("t").alias("tib"),
                    PossibleValue::new("kb"),
                    PossibleValue::new("mb"),
                    PossibleValue::new("gb"),
                    PossibleValue::new("tb"),
                ])
                .ignore_case(true)
                .help("Changes output display size. si will print sizes in powers of 1000. b k m g t kb mb gb tb will print the whole tree in that size.")
        )
        .arg(
            Arg::new("stack_size")
                .short('S')
                .long("stack-size")
                .value_name("STACK_SIZE")
                .value_parser(value_parser!(usize))
                .num_args(1)
                .help("Specify memory to use as stack size - use if you see: 'fatal runtime error: stack overflow' (default low memory=1048576, high memory=1073741824)"),
        )
        .arg(
            Arg::new("params")
                .value_name("PATH")
                .value_hint(clap::ValueHint::AnyPath)
                .value_parser(value_parser!(String))
                .num_args(1..)
        )
        .arg(
            Arg::new("output_json")
                .short('j')
                .long("output-json")
                .action(clap::ArgAction::SetTrue)
                .help("Output the directory tree as json to the current directory"),
        )
        .arg(
            Arg::new("mtime")
            .short('M')
            .long("mtime")
            .num_args(1)
            .allow_hyphen_values(true)
            .value_parser(value_parser!(String))
            .help("+/-n matches files modified more/less than n days ago , and n matches files modified exactly n days ago, days are rounded down.That is +n => (−∞, curr−(n+1)), n => [curr−(n+1), curr−n), and -n => (𝑐𝑢𝑟𝑟−𝑛, +∞)")
        )
        .arg(
            Arg::new("atime")
            .short('A')
            .long("atime")
            .num_args(1)
            .allow_hyphen_values(true)
            .value_parser(value_parser!(String))
            .help("just like -mtime, but based on file access time")
        )
        .arg(
            Arg::new("ctime")
            .short('y')
            .long("ctime")
            .num_args(1)
            .allow_hyphen_values(true)
            .value_parser(value_parser!(String))
            .help("just like -mtime, but based on file change time")
        )
        .arg(
            Arg::new("files0_from")
                .long("files0-from")
                .value_hint(clap::ValueHint::AnyPath)
                .value_parser(value_parser!(String))
                .num_args(1)
                .help("run dust on NUL-terminated file names specified in file; if argument is -, then read names from standard input"),
        )
        .arg(
            Arg::new("collapse")
                .long("collapse")
                .value_hint(clap::ValueHint::AnyPath)
                .value_parser(value_parser!(String))
                .action(clap::ArgAction::Append)
                .help("Keep these directories collapsed"),
        )
        .arg(
            Arg::new("filetime")
                .short('m')
                .long("filetime")
                .num_args(1)
                .value_parser([
                    PossibleValue::new("a").alias("accessed"),
                    PossibleValue::new("c").alias("changed"),
                    PossibleValue::new("m").alias("modified"),
                ])
                .help("Directory 'size' is max filetime of child files instead of disk size. while a/c/m for last accessed/changed/modified time"),
        )
}
