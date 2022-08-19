use clap::ArgMatches;
use config_file::FromConfigFile;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

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
    pub ignore_hidden: Option<bool>,
    pub iso: Option<bool>,
}

impl Config {
    pub fn get_no_colors(&self, options: &ArgMatches) -> bool {
        Some(true) == self.no_colors || options.is_present("no_colors")
    }
    pub fn get_apparent_size(&self, options: &ArgMatches) -> bool {
        Some(true) == self.display_apparent_size || options.is_present("display_apparent_size")
    }
    pub fn get_ignore_hidden(&self, options: &ArgMatches) -> bool {
        Some(true) == self.ignore_hidden || options.is_present("ignore_hidden")
    }
    pub fn get_full_paths(&self, options: &ArgMatches) -> bool {
        Some(true) == self.display_full_paths || options.is_present("display_full_paths")
    }
    pub fn get_reverse(&self, options: &ArgMatches) -> bool {
        Some(true) == self.reverse || options.is_present("reverse")
    }
    pub fn get_no_bars(&self, options: &ArgMatches) -> bool {
        Some(true) == self.no_bars || options.is_present("no_bars")
    }
    pub fn get_iso(&self, options: &ArgMatches) -> bool {
        Some(true) == self.iso || options.is_present("iso")
    }
    pub fn get_skip_total(&self, options: &ArgMatches) -> bool {
        Some(true) == self.skip_total || options.is_present("skip_total")
    }
}

fn get_config_locations(base: &Path) -> Vec<PathBuf> {
    vec![
        base.join(".dust.toml"),
        base.join(".dust.yaml"),
        base.join(".config").join("dust").join("config.toml"),
        base.join(".config").join("dust").join("config.yaml"),
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
