use clap::ArgMatches;
use config_file::FromConfigFile;
use serde::Deserialize;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;

use crate::display::UNITS;

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub display_full_paths: Option<bool>,
    pub display_apparent_size: Option<bool>,
    pub reverse: Option<bool>,
    pub no_colors: Option<bool>,
    pub no_bars: Option<bool>,
    pub skip_total: Option<bool>,
    pub screen_reader: Option<bool>,
    pub ignore_hidden: Option<bool>,
    pub iso: Option<bool>,
    pub min_size: Option<String>,
    pub only_dir: Option<bool>,
    pub only_file: Option<bool>,
    pub disable_progress: Option<bool>,
    pub depth: Option<usize>,
    pub bars_on_right: Option<bool>,
    pub stack_size: Option<usize>,
}

impl Config {
    pub fn get_no_colors(&self, options: &ArgMatches) -> bool {
        Some(true) == self.no_colors || options.get_flag("no_colors")
    }
    pub fn get_disable_progress(&self, options: &ArgMatches) -> bool {
        Some(true) == self.disable_progress
            || options.get_flag("disable_progress")
            || !std::io::stdout().is_terminal()
    }
    pub fn get_apparent_size(&self, options: &ArgMatches) -> bool {
        Some(true) == self.display_apparent_size || options.get_flag("display_apparent_size")
    }
    pub fn get_ignore_hidden(&self, options: &ArgMatches) -> bool {
        Some(true) == self.ignore_hidden || options.get_flag("ignore_hidden")
    }
    pub fn get_full_paths(&self, options: &ArgMatches) -> bool {
        // If we are only showing files, always show full paths
        Some(true) == self.display_full_paths
            || options.get_flag("display_full_paths")
            || self.get_only_file(options)
    }
    pub fn get_reverse(&self, options: &ArgMatches) -> bool {
        Some(true) == self.reverse || options.get_flag("reverse")
    }
    pub fn get_no_bars(&self, options: &ArgMatches) -> bool {
        Some(true) == self.no_bars || options.get_flag("no_bars")
    }
    pub fn get_iso(&self, options: &ArgMatches) -> bool {
        Some(true) == self.iso || options.get_flag("iso")
    }
    pub fn get_skip_total(&self, options: &ArgMatches) -> bool {
        Some(true) == self.skip_total || options.get_flag("skip_total")
    }
    pub fn get_screen_reader(&self, options: &ArgMatches) -> bool {
        Some(true) == self.screen_reader || options.get_flag("screen_reader")
    }
    pub fn get_depth(&self, options: &ArgMatches) -> usize {
        if let Some(v) = options.get_one::<usize>("depth") {
            return *v;
        }

        self.depth.unwrap_or(usize::MAX)
    }
    pub fn get_min_size(&self, options: &ArgMatches, iso: bool) -> Option<usize> {
        let size_from_param = options.get_one::<String>("min_size");
        self._get_min_size(size_from_param, iso)
    }
    fn _get_min_size(&self, min_size: Option<&String>, iso: bool) -> Option<usize> {
        let size_from_param = min_size.and_then(|a| convert_min_size(a, iso));

        if size_from_param.is_none() {
            self.min_size
                .as_ref()
                .and_then(|a| convert_min_size(a.as_ref(), iso))
        } else {
            size_from_param
        }
    }
    pub fn get_only_dir(&self, options: &ArgMatches) -> bool {
        Some(true) == self.only_dir || options.get_flag("only_dir")
    }
    pub fn get_only_file(&self, options: &ArgMatches) -> bool {
        Some(true) == self.only_file || options.get_flag("only_file")
    }
    pub fn get_bars_on_right(&self, options: &ArgMatches) -> bool {
        Some(true) == self.bars_on_right || options.get_flag("bars_on_right")
    }
    pub fn get_custom_stack_size(&self, options: &ArgMatches) -> Option<usize> {
        let from_cmd_line = options.get_one::<usize>("stack_size");
        if from_cmd_line.is_none() {
            self.stack_size
        } else {
            from_cmd_line.copied()
        }
    }
}

fn convert_min_size(input: &str, iso: bool) -> Option<usize> {
    let chars_as_vec: Vec<char> = input.chars().collect();
    match chars_as_vec.split_last() {
        Some((last, start)) => {
            let mut starts: String = start.iter().collect::<String>();

            for (i, u) in UNITS.iter().rev().enumerate() {
                if Some(*u) == last.to_uppercase().next() {
                    return match starts.parse::<usize>() {
                        Ok(pure) => {
                            let num: usize = if iso { 1000 } else { 1024 };
                            let marker = pure * num.pow((i + 1) as u32);
                            Some(marker)
                        }
                        Err(_) => {
                            eprintln!("Ignoring invalid min-size: {input}");
                            None
                        }
                    };
                }
            }
            starts.push(*last);
            starts
                .parse()
                .map_err(|_| {
                    eprintln!("Ignoring invalid min-size: {input}");
                })
                .ok()
        }
        None => None,
    }
}

fn get_config_locations(base: &Path) -> Vec<PathBuf> {
    vec![
        base.join(".dust.toml"),
        base.join(".config").join("dust").join("config.toml"),
    ]
}

pub fn get_config() -> Config {
    if let Some(home) = directories::BaseDirs::new() {
        for path in get_config_locations(home.home_dir()) {
            if path.exists() {
                if let Ok(config) = Config::from_config_file(path) {
                    return config;
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
    use clap::{value_parser, Arg, ArgMatches, Command};

    #[test]
    fn test_conversion() {
        assert_eq!(convert_min_size("55", false), Some(55));
        assert_eq!(convert_min_size("12344321", false), Some(12344321));
        assert_eq!(convert_min_size("95RUBBISH", false), None);
        assert_eq!(convert_min_size("10K", false), Some(10 * 1024));
        assert_eq!(convert_min_size("10M", false), Some(10 * 1024usize.pow(2)));
        assert_eq!(convert_min_size("10M", true), Some(10 * 1000usize.pow(2)));
        assert_eq!(convert_min_size("2G", false), Some(2 * 1024usize.pow(3)));
    }

    #[test]
    fn test_min_size_from_config_applied_or_overridden() {
        let c = Config {
            min_size: Some("1K".to_owned()),
            ..Default::default()
        };
        assert_eq!(c._get_min_size(None, false), Some(1024));
        assert_eq!(c._get_min_size(Some(&"2K".into()), false), Some(2048));

        assert_eq!(c._get_min_size(None, true), Some(1000));
        assert_eq!(c._get_min_size(Some(&"2K".into()), true), Some(2000));
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

    fn get_args(args: Vec<&str>) -> ArgMatches {
        Command::new("Dust")
            .arg(
                Arg::new("depth")
                    .long("depth")
                    .num_args(1)
                    .value_parser(value_parser!(usize)),
            )
            .get_matches_from(args)
    }
}
