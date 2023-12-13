use clap::{value_parser, Arg, Command};

// For single thread mode set this variable on your command line:
// export RAYON_NUM_THREADS=1

pub fn build_cli() -> Command {
    Command::new("Dust")
        .about("Like du but more intuitive")
        .version(env!("CARGO_PKG_VERSION"))
        .trailing_var_arg(true)
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .value_parser(value_parser!(usize))
                .help("Depth to show")
                .num_args(1)
        )
        .arg(
            Arg::new("number_of_lines")
                .short('n')
                .long("number-of-lines")
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
                .action(clap::ArgAction::Append)
                .help("Exclude any file or directory with this name"),
        )
        .arg(
            Arg::new("ignore_all_in_file")
                .short('I')
                .long("ignore-all-in-file")
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
                .action(clap::ArgAction::Append)
                .conflicts_with("filter")
                .conflicts_with("types")
                .help("Exclude filepaths matching this regex. To ignore png files type: -v \"\\.png$\" "),
        )
        .arg(
            Arg::new("filter")
                .short('e')
                .long("filter")
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
                .num_args(1)
                .value_parser(value_parser!(usize))
                .help("Specify width of output overriding the auto detection of terminal width"),
        )
        .arg(
            Arg::new("iso")
                .short('H')
                .long("si")
                .action(clap::ArgAction::SetTrue)
                .help("print sizes in powers of 1000 (e.g., 1.1G)")
        )
        .arg(
            Arg::new("disable_progress")
                .short('P')
                .long("no-progress")
                .action(clap::ArgAction::SetTrue)
                .help("Disable the progress indication."),
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
            Arg::new("stack_size")
                .short('S')
                .long("stack-size")
                .num_args(1)
                .value_parser(value_parser!(usize))
                .help("Specify memory to use as stack size - use if you see: 'fatal runtime error: stack overflow' (default low memory=1048576, high memory=1073741824)"),
        )
        .arg(Arg::new("params").num_args(1..)
            .value_parser(value_parser!(String)))
}
