use crate::node::FileTime;
use chrono::{Local, TimeZone};
use config_file::FromConfigFile;
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::dir_walker::Operator;
use crate::display::get_number_format;

pub static DAY_SECONDS: i64 = 24 * 60 * 60;

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub display_full_paths: Option<bool>,
    pub display_apparent_size: Option<bool>,
    pub reverse: Option<bool>,
    pub no_colors: Option<bool>,
    pub force_colors: Option<bool>,
    pub no_bars: Option<bool>,
    pub skip_total: Option<bool>,
    pub screen_reader: Option<bool>,
    pub ignore_hidden: Option<bool>,
    pub output_format: Option<String>,
    pub min_size: Option<String>,
    pub only_dir: Option<bool>,
    pub only_file: Option<bool>,
    pub disable_progress: Option<bool>,
    pub depth: Option<usize>,
    pub bars_on_right: Option<bool>,
    pub stack_size: Option<usize>,
    pub threads: Option<usize>,
    pub output_json: Option<bool>,
    pub print_errors: Option<bool>,
    pub files0_from: Option<String>,
    pub number_of_lines: Option<usize>,
    pub files_from: Option<String>,
}

impl Config {
    pub fn get_files0_from(&self, options: &Cli) -> Option<String> {
        let from_file = &options.files0_from;
        match from_file {
            None => self.files0_from.as_ref().map(|x| x.to_string()),
            Some(x) => Some(x.to_string()),
        }
    }

    pub fn get_files_from(&self, options: &Cli) -> Option<String> {
        let from_file = &options.files_from;
        match from_file {
            None => self.files_from.as_ref().map(|x| x.to_string()),
            Some(x) => Some(x.to_string()),
        }
    }
    pub fn get_no_colors(&self, options: &Cli) -> bool {
        Some(true) == self.no_colors || options.no_colors
    }
    pub fn get_force_colors(&self, options: &Cli) -> bool {
        Some(true) == self.force_colors || options.force_colors
    }
    pub fn get_disable_progress(&self, options: &Cli) -> bool {
        Some(true) == self.disable_progress || options.no_progress
    }
    pub fn get_apparent_size(&self, options: &Cli) -> bool {
        Some(true) == self.display_apparent_size || options.apparent_size
    }
    pub fn get_ignore_hidden(&self, options: &Cli) -> bool {
        Some(true) == self.ignore_hidden || options.ignore_hidden
    }
    pub fn get_full_paths(&self, options: &Cli) -> bool {
        Some(true) == self.display_full_paths || options.full_paths
    }
    pub fn get_reverse(&self, options: &Cli) -> bool {
        Some(true) == self.reverse || options.reverse
    }
    pub fn get_no_bars(&self, options: &Cli) -> bool {
        Some(true) == self.no_bars || options.no_percent_bars
    }
    pub fn get_output_format(&self, options: &Cli) -> String {
        let out_fmt = options.output_format;
        (match out_fmt {
            None => match &self.output_format {
                None => "".to_string(),
                Some(x) => x.to_string(),
            },
            Some(x) => x.to_string(),
        })
        .to_lowercase()
    }

    pub fn get_filetime(&self, options: &Cli) -> Option<FileTime> {
        options.filetime.map(FileTime::from)
    }

    pub fn get_skip_total(&self, options: &Cli) -> bool {
        Some(true) == self.skip_total || options.skip_total
    }
    pub fn get_screen_reader(&self, options: &Cli) -> bool {
        Some(true) == self.screen_reader || options.screen_reader
    }
    pub fn get_depth(&self, options: &Cli) -> usize {
        if let Some(v) = options.depth {
            return v;
        }

        self.depth.unwrap_or(usize::MAX)
    }
    pub fn get_min_size(&self, options: &Cli) -> Option<usize> {
        let size_from_param = options.min_size.as_ref();
        self._get_min_size(size_from_param)
    }
    fn _get_min_size(&self, min_size: Option<&String>) -> Option<usize> {
        let size_from_param = min_size.and_then(|a| convert_min_size(a));

        if size_from_param.is_none() {
            self.min_size
                .as_ref()
                .and_then(|a| convert_min_size(a.as_ref()))
        } else {
            size_from_param
        }
    }
    pub fn get_only_dir(&self, options: &Cli) -> bool {
        Some(true) == self.only_dir || options.only_dir
    }

    pub fn get_print_errors(&self, options: &Cli) -> bool {
        Some(true) == self.print_errors || options.print_errors
    }
    pub fn get_only_file(&self, options: &Cli) -> bool {
        Some(true) == self.only_file || options.only_file
    }
    pub fn get_bars_on_right(&self, options: &Cli) -> bool {
        Some(true) == self.bars_on_right || options.bars_on_right
    }
    pub fn get_custom_stack_size(&self, options: &Cli) -> Option<usize> {
        let from_cmd_line = options.stack_size;
        if from_cmd_line.is_none() {
            self.stack_size
        } else {
            from_cmd_line
        }
    }
    pub fn get_threads(&self, options: &Cli) -> Option<usize> {
        let from_cmd_line = options.threads;
        if from_cmd_line.is_none() {
            self.threads
        } else {
            from_cmd_line
        }
    }
    pub fn get_output_json(&self, options: &Cli) -> bool {
        Some(true) == self.output_json || options.output_json
    }

    pub fn get_number_of_lines(&self, options: &Cli) -> Option<usize> {
        let from_cmd_line = options.number_of_lines;
        if from_cmd_line.is_none() {
            self.number_of_lines
        } else {
            from_cmd_line
        }
    }

    pub fn get_modified_time_operator(&self, options: &Cli) -> Option<(Operator, i64)> {
        get_filter_time_operator(options.mtime.as_ref(), get_current_date_epoch_seconds())
    }

    pub fn get_accessed_time_operator(&self, options: &Cli) -> Option<(Operator, i64)> {
        get_filter_time_operator(options.atime.as_ref(), get_current_date_epoch_seconds())
    }

    pub fn get_changed_time_operator(&self, options: &Cli) -> Option<(Operator, i64)> {
        get_filter_time_operator(options.ctime.as_ref(), get_current_date_epoch_seconds())
    }
}

fn get_current_date_epoch_seconds() -> i64 {
    // calculate current date epoch seconds
    let now = Local::now();
    let current_date = now.date_naive();

    let current_date_time = current_date.and_hms_opt(0, 0, 0).unwrap();
    Local
        .from_local_datetime(&current_date_time)
        .unwrap()
        .timestamp()
}

fn get_filter_time_operator(
    option_value: Option<&String>,
    current_date_epoch_seconds: i64,
) -> Option<(Operator, i64)> {
    match option_value {
        Some(val) => {
            let time = current_date_epoch_seconds
                - val
                    .parse::<i64>()
                    .unwrap_or_else(|_| panic!("invalid data format"))
                    .abs()
                    * DAY_SECONDS;
            match val.chars().next().expect("Value should not be empty") {
                '+' => Some((Operator::LessThan, time - DAY_SECONDS)),
                '-' => Some((Operator::GreaterThan, time)),
                _ => Some((Operator::Equal, time - DAY_SECONDS)),
            }
        }
        None => None,
    }
}

fn convert_min_size(input: &str) -> Option<usize> {
    let re = Regex::new(r"([0-9]+)(\w*)").unwrap();

    if let Some(cap) = re.captures(input) {
        let (_, [digits, letters]) = cap.extract();

        // Failure to parse should be impossible due to regex match
        let digits_as_usize: Option<usize> = digits.parse().ok();

        match digits_as_usize {
            Some(parsed_digits) => {
                let number_format = get_number_format(&letters.to_lowercase());
                match number_format {
                    Some((multiple, _)) => Some(parsed_digits * (multiple as usize)),
                    None => {
                        if letters.is_empty() {
                            Some(parsed_digits)
                        } else {
                            eprintln!("Ignoring invalid min-size: {input}");
                            None
                        }
                    }
                }
            }
            None => None,
        }
    } else {
        None
    }
}

fn get_config_locations(base: PathBuf) -> Vec<PathBuf> {
    vec![
        base.join(".dust.toml"),
        base.join(".config").join("dust").join("config.toml"),
    ]
}

pub fn get_config(conf_path: Option<&String>) -> Config {
    match conf_path {
        Some(path_str) => {
            let path = Path::new(path_str);
            if path.exists() {
                match Config::from_config_file(path) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Ignoring invalid config file '{}': {}", &path.display(), e)
                    }
                }
            } else {
                eprintln!("Config file {:?} doesn't exists", &path.display());
            }
        }
        None => {
            if let Some(home) = std::env::home_dir() {
                for path in get_config_locations(home) {
                    if path.exists()
                        && let Ok(config) = Config::from_config_file(&path)
                    {
                        return config;
                    }
                }
            }
        }
    }
    Config {
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use chrono::{Datelike, Timelike};
    use clap::Parser;

    #[test]
    fn test_get_current_date_epoch_seconds() {
        let epoch_seconds = get_current_date_epoch_seconds();
        let dt = Local.timestamp_opt(epoch_seconds, 0).unwrap();

        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
        assert_eq!(dt.date_naive().day(), Local::now().date_naive().day());
        assert_eq!(dt.date_naive().month(), Local::now().date_naive().month());
        assert_eq!(dt.date_naive().year(), Local::now().date_naive().year());
    }

    #[test]
    fn test_conversion() {
        assert_eq!(convert_min_size("55"), Some(55));
        assert_eq!(convert_min_size("12344321"), Some(12344321));
        assert_eq!(convert_min_size("95RUBBISH"), None);
        assert_eq!(convert_min_size("10Ki"), Some(10 * 1024));
        assert_eq!(convert_min_size("10MiB"), Some(10 * 1024usize.pow(2)));
        assert_eq!(convert_min_size("10M"), Some(10 * 1024usize.pow(2)));
        assert_eq!(convert_min_size("10Mb"), Some(10 * 1000usize.pow(2)));
        assert_eq!(convert_min_size("2Gi"), Some(2 * 1024usize.pow(3)));
    }

    #[test]
    fn test_min_size_from_config_applied_or_overridden() {
        let c = Config {
            min_size: Some("1KiB".to_owned()),
            ..Default::default()
        };
        assert_eq!(c._get_min_size(None), Some(1024));
        assert_eq!(c._get_min_size(Some(&"2KiB".into())), Some(2048));

        assert_eq!(c._get_min_size(Some(&"1kb".into())), Some(1000));
        assert_eq!(c._get_min_size(Some(&"2KB".into())), Some(2000));
    }

    #[test]
    fn test_get_depth() {
        // No config and no flag.
        let c = Config::default();
        let args = get_args(vec![]);
        assert_eq!(c.get_depth(&args), usize::MAX);

        // Config is not defined and flag is defined.
        let c = Config::default();
        let args = get_args(vec!["dust", "--depth", "5"]);
        assert_eq!(c.get_depth(&args), 5);

        // Config is defined and flag is not defined.
        let c = Config {
            depth: Some(3),
            ..Default::default()
        };
        let args = get_args(vec![]);
        assert_eq!(c.get_depth(&args), 3);

        // Both config and flag are defined.
        let c = Config {
            depth: Some(3),
            ..Default::default()
        };
        let args = get_args(vec!["dust", "--depth", "5"]);
        assert_eq!(c.get_depth(&args), 5);
    }

    fn get_args(args: Vec<&str>) -> Cli {
        Cli::parse_from(args)
    }

    #[test]
    fn test_get_filetime() {
        // No config and no flag.
        let c = Config::default();
        let args = get_filetime_args(vec!["dust"]);
        assert_eq!(c.get_filetime(&args), None);

        // Config is not defined and flag is defined as access time
        let c = Config::default();
        let args = get_filetime_args(vec!["dust", "--filetime", "a"]);
        assert_eq!(c.get_filetime(&args), Some(FileTime::Accessed));

        let c = Config::default();
        let args = get_filetime_args(vec!["dust", "--filetime", "accessed"]);
        assert_eq!(c.get_filetime(&args), Some(FileTime::Accessed));

        // Config is not defined and flag is defined as modified time
        let c = Config::default();
        let args = get_filetime_args(vec!["dust", "--filetime", "m"]);
        assert_eq!(c.get_filetime(&args), Some(FileTime::Modified));

        let c = Config::default();
        let args = get_filetime_args(vec!["dust", "--filetime", "modified"]);
        assert_eq!(c.get_filetime(&args), Some(FileTime::Modified));

        // Config is not defined and flag is defined as changed time
        let c = Config::default();
        let args = get_filetime_args(vec!["dust", "--filetime", "c"]);
        assert_eq!(c.get_filetime(&args), Some(FileTime::Changed));

        let c = Config::default();
        let args = get_filetime_args(vec!["dust", "--filetime", "changed"]);
        assert_eq!(c.get_filetime(&args), Some(FileTime::Changed));
    }

    fn get_filetime_args(args: Vec<&str>) -> Cli {
        Cli::parse_from(args)
    }

    #[test]
    fn test_get_number_of_lines() {
        // No config and no flag.
        let c = Config::default();
        let args = get_args(vec![]);
        assert_eq!(c.get_number_of_lines(&args), None);

        // Config is not defined and flag is defined.
        let c = Config::default();
        let args = get_args(vec!["dust", "--number-of-lines", "5"]);
        assert_eq!(c.get_number_of_lines(&args), Some(5));

        // Config is defined and flag is not defined.
        let c = Config {
            number_of_lines: Some(3),
            ..Default::default()
        };
        let args = get_args(vec![]);
        assert_eq!(c.get_number_of_lines(&args), Some(3));

        // Both config and flag are defined.
        let c = Config {
            number_of_lines: Some(3),
            ..Default::default()
        };
        let args = get_args(vec!["dust", "--number-of-lines", "5"]);
        assert_eq!(c.get_number_of_lines(&args), Some(5));
    }
}
