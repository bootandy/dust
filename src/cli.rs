use std::fmt;

use clap::{Parser, ValueEnum, ValueHint};

// For single thread mode set this variable on your command line:
// export RAYON_NUM_THREADS=1

/// Like du but more intuitive
#[derive(Debug, Parser)]
#[command(name("Dust"), version)]
pub struct Cli {
    /// Depth to show
    #[arg(short, long)]
    pub depth: Option<usize>,

    /// Number of threads to use
    #[arg(short('T'), long)]
    pub threads: Option<usize>,

    /// Specify a config file to use
    #[arg(long, value_name("FILE"), value_hint(ValueHint::FilePath))]
    pub config: Option<String>,

    /// Number of lines of output to show. (Default is terminal_height - 10)
    #[arg(short, long, value_name("NUMBER"))]
    pub number_of_lines: Option<usize>,

    /// Subdirectories will not have their path shortened
    #[arg(short('p'), long)]
    pub full_paths: bool,

    /// Exclude any file or directory with this path
    #[arg(short('X'), long, value_name("PATH"), value_hint(ValueHint::AnyPath))]
    pub ignore_directory: Option<Vec<String>>,

    /// Exclude any file or directory with a regex matching that listed in this
    /// file, the file entries will be added to the ignore regexs provided by
    /// --invert_filter
    #[arg(short('I'), long, value_name("FILE"), value_hint(ValueHint::FilePath))]
    pub ignore_all_in_file: Option<String>,

    /// dereference sym links - Treat sym links as directories and go into them
    #[arg(short('L'), long)]
    pub dereference_links: bool,

    /// Only count the files and directories on the same filesystem as the
    /// supplied directory
    #[arg(short('x'), long)]
    pub limit_filesystem: bool,

    /// Use file length instead of blocks
    #[arg(short('s'), long)]
    pub apparent_size: bool,

    /// Print tree upside down (biggest highest)
    #[arg(short, long)]
    pub reverse: bool,

    /// No colors will be printed (Useful for commands like: watch)
    #[arg(short('c'), long)]
    pub no_colors: bool,

    /// Force colors print
    #[arg(short('C'), long)]
    pub force_colors: bool,

    /// No percent bars or percentages will be displayed
    #[arg(short('b'), long)]
    pub no_percent_bars: bool,

    /// percent bars moved to right side of screen
    #[arg(short('B'), long)]
    pub bars_on_right: bool,

    /// Minimum size file to include in output
    #[arg(short('z'), long)]
    pub min_size: Option<String>,

    /// For screen readers. Removes bars. Adds new column: depth level (May want
    /// to use -p too for full path)
    #[arg(short('R'), long)]
    pub screen_reader: bool,

    /// No total row will be displayed
    #[arg(long)]
    pub skip_total: bool,

    /// Directory 'size' is number of child files instead of disk size
    #[arg(short, long)]
    pub filecount: bool,

    /// Do not display hidden files
    // Do not use 'h' this is used by 'help'
    #[arg(short, long)]
    pub ignore_hidden: bool,

    /// Exclude filepaths matching this regex. To ignore png files type: -v
    /// "\.png$"
    #[arg(
        short('v'),
        long,
        value_name("REGEX"),
        conflicts_with("filter"),
        conflicts_with("file_types")
    )]
    pub invert_filter: Option<Vec<String>>,

    /// Only include filepaths matching this regex. For png files type: -e
    /// "\.png$"
    #[arg(short('e'), long, value_name("REGEX"), conflicts_with("file_types"))]
    pub filter: Option<Vec<String>>,

    /// show only these file types
    #[arg(short('t'), long, conflicts_with("depth"), conflicts_with("only_dir"))]
    pub file_types: bool,

    /// Specify width of output overriding the auto detection of terminal width
    #[arg(short('w'), long, value_name("WIDTH"))]
    pub terminal_width: Option<usize>,

    /// Disable the progress indication.
    #[arg(short('P'), long)]
    pub no_progress: bool,

    /// Print path with errors.
    #[arg(long)]
    pub print_errors: bool,

    /// Only directories will be displayed.
    #[arg(
        short('D'),
        long,
        conflicts_with("only_file"),
        conflicts_with("file_types")
    )]
    pub only_dir: bool,

    /// Only files will be displayed. (Finds your largest files)
    #[arg(short('F'), long, conflicts_with("only_dir"))]
    pub only_file: bool,

    /// Changes output display size. si will print sizes in powers of 1000. b k
    /// m g t kb mb gb tb will print the whole tree in that size.
    #[arg(short, long, value_enum, value_name("FORMAT"), ignore_case(true))]
    pub output_format: Option<OutputFormat>,

    /// Specify memory to use as stack size - use if you see: 'fatal runtime
    /// error: stack overflow' (default low memory=1048576, high
    /// memory=1073741824)
    #[arg(short('S'), long)]
    pub stack_size: Option<usize>,

    /// Input files or directories.
    #[arg(value_name("PATH"), value_hint(ValueHint::AnyPath))]
    pub params: Option<Vec<String>>,

    /// Output the directory tree as json to the current directory
    #[arg(short('j'), long)]
    pub output_json: bool,

    /// +/-n matches files modified more/less than n days ago , and n matches
    /// files modified exactly n days ago, days are rounded down.That is +n =>
    /// (−∞, curr−(n+1)), n => [curr−(n+1), curr−n), and -n => (𝑐𝑢𝑟𝑟−𝑛, +∞)
    #[arg(short('M'), long, allow_hyphen_values(true))]
    pub mtime: Option<String>,

    /// just like -mtime, but based on file access time
    #[arg(short('A'), long, allow_hyphen_values(true))]
    pub atime: Option<String>,

    /// just like -mtime, but based on file change time
    #[arg(short('y'), long, allow_hyphen_values(true))]
    pub ctime: Option<String>,

    /// run dust on NUL-terminated file names specified in file; if argument is
    /// -, then read names from standard input
    #[arg(long, value_hint(ValueHint::AnyPath))]
    pub files0_from: Option<String>,

    /// Keep these directories collapsed
    #[arg(long, value_hint(ValueHint::AnyPath))]
    pub collapse: Option<Vec<String>>,

    /// Directory 'size' is max filetime of child files instead of disk size.
    /// while a/c/m for last accessed/changed/modified time
    #[arg(short('m'), long, value_enum)]
    pub filetime: Option<FileTime>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "lower")]
pub enum OutputFormat {
    /// SI prefix (powers of 1000)
    SI,

    /// byte (B)
    B,

    /// kibibyte (KiB)
    #[value(name = "k", alias("kib"))]
    KiB,

    /// mebibyte (MiB)
    #[value(name = "m", alias("mib"))]
    MiB,

    /// gibibyte (GiB)
    #[value(name = "g", alias("gib"))]
    GiB,

    /// tebibyte (TiB)
    #[value(name = "t", alias("tib"))]
    TiB,

    /// kilobyte (kB)
    KB,

    /// megabyte (MB)
    MB,

    /// gigabyte (GB)
    GB,

    /// terabyte (TB)
    TB,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SI => write!(f, "si"),
            Self::B => write!(f, "b"),
            Self::KiB => write!(f, "k"),
            Self::MiB => write!(f, "m"),
            Self::GiB => write!(f, "g"),
            Self::TiB => write!(f, "t"),
            Self::KB => write!(f, "kb"),
            Self::MB => write!(f, "mb"),
            Self::GB => write!(f, "gb"),
            Self::TB => write!(f, "tb"),
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FileTime {
    /// last accessed time
    #[value(name = "a", alias("accessed"))]
    Accessed,

    /// last changed time
    #[value(name = "c", alias("changed"))]
    Changed,

    /// last modified time
    #[value(name = "m", alias("modified"))]
    Modified,
}
