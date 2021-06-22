use assert_cmd::Command;
use std::str;
use std::sync::Once;

static INIT: Once = Once::new();

mod tests_symlinks;

/**
 * This file contains tests that verify the exact output of the command.
 * This output differs on Linux / Mac so the tests are harder to write and debug
 * Windows is ignored here because the results vary by host making exact testing impractical
 *
 * Despite the above problems, these tests are good as they are the closest to 'the real thing'.
 */

//  Warning: File sizes differ on both platform and on the format of the disk.
/// Copy to /tmp dir - we assume that the formatting of the /tmp partition
/// is consistent. If the tests fail your /tmp filesystem probably differs
fn copy_test_data(dir: &str) {
    // First remove the existing directory - just incase it is there and has incorrect data
    let last_slash = dir.rfind('/').unwrap();
    let last_part_of_dir = dir.chars().skip(last_slash).collect::<String>();
    match Command::new("rm")
        .arg("-rf")
        .arg("/tmp/".to_owned() + &*last_part_of_dir)
        .ok()
    {
        Ok(_) => {}
        Err(_) => {}
    };
    match Command::new("cp").arg("-r").arg(dir).arg("/tmp/").ok() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error copying directory {:?}", err);
        }
    };
}

pub fn initialize() {
    INIT.call_once(|| {
        copy_test_data("tests/test_dir");
        copy_test_data("tests/test_dir2");
        copy_test_data("tests/test_dir_unicode");
    });
}

// "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_basic() {
    // -c is no color mode - This makes testing much simpler
    initialize();
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let assert = cmd.arg("-c").arg("/tmp/test_dir/").unwrap().stdout;
    let output = str::from_utf8(&assert).unwrap();
    assert!(output.contains(&main_output()));
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_multi_arg() {
    initialize();
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let assert = cmd
        .arg("-c")
        .arg("/tmp/test_dir/many/")
        .arg("/tmp/test_dir")
        .arg("/tmp/test_dir")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&assert).unwrap();
    assert!(output.contains(&main_output()));
}

#[cfg(target_os = "macos")]
fn main_output() -> String {
    r#"
   0B     ┌── a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── hello_file│████████████████████████████████████████████████ │ 100%
 4.0K   ┌─┴ many        │████████████████████████████████████████████████ │ 100%
 4.0K ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%
 "#
    .trim()
    .to_string()
}

#[cfg(target_os = "linux")]
fn main_output() -> String {
    r#"
   0B     ┌── a_file    │               ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── hello_file│               ░░░░░░░░░░░░░░░░█████████████████ │  33%
 8.0K   ┌─┴ many        │               █████████████████████████████████ │  67%
  12K ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%
  "#
    .trim()
    .to_string()
}

#[cfg(target_os = "windows")]
fn main_output() -> String {
    "windows results vary by host".to_string()
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_long_paths() {
    initialize();
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let assert = cmd
        .arg("-c")
        .arg("-p")
        .arg("/tmp/test_dir/")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&assert).unwrap();
    assert!(output.contains(&main_output_long_paths()));
}

#[cfg(target_os = "macos")]
fn main_output_long_paths() -> String {
    r#"
   0B     ┌── /tmp/test_dir/many/a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── /tmp/test_dir/many/hello_file│█████████████████████████████ │ 100%
 4.0K   ┌─┴ /tmp/test_dir/many             │█████████████████████████████ │ 100%
 4.0K ┌─┴ /tmp/test_dir                    │█████████████████████████████ │ 100%
"#
    .trim()
    .to_string()
}

#[cfg(target_os = "linux")]
fn main_output_long_paths() -> String {
    r#"   
   0B     ┌── /tmp/test_dir/many/a_file    │         ░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── /tmp/test_dir/many/hello_file│         ░░░░░░░░░░██████████ │  33%
 8.0K   ┌─┴ /tmp/test_dir/many             │         ████████████████████ │  67%
  12K ┌─┴ /tmp/test_dir                    │█████████████████████████████ │ 100%
"#
    .trim()
    .to_string()
}

#[cfg(target_os = "windows")]
fn main_output_long_paths() -> String {
    "windows results vary by host".to_string()
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_apparent_size() {
    initialize();
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let assert = cmd.arg("-c").arg("-s").arg("/tmp/test_dir").unwrap().stdout;
    let output = str::from_utf8(&assert).unwrap();
    assert!(output.contains(&output_apparent_size()));
}

#[cfg(target_os = "linux")]
fn output_apparent_size() -> String {
    r#"
   0B     ┌── a_file    │                       ░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
   6B     ├── hello_file│                       ░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K   ┌─┴ many        │                       █████████████████████████ │  50%
 8.0K ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%     
"#
    .trim()
    .to_string()
}

#[cfg(target_os = "macos")]
fn output_apparent_size() -> String {
    r#"
   0B     ┌── a_file    │                    ░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
   6B     ├── hello_file│                    ░░░░░░░░░░░░░░░░░░░░░░░░░░██ │   3%
 134B   ┌─┴ many        │                    ████████████████████████████ │  58%
 230B ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%
"#
    .trim()
    .to_string()
}

#[cfg(target_os = "windows")]
fn output_apparent_size() -> String {
    "windows results vary by host".to_string()
}

// Check against directories and files whos names are substrings of each other
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_substring_of_names_and_long_names() {
    initialize();
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd.arg("-c").arg("/tmp/test_dir2").unwrap().stdout;
    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains(&no_substring_of_names_output()));
}

#[cfg(target_os = "linux")]
fn no_substring_of_names_output() -> String {
    "
   0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_g..
 4.0K   ├── dir_name_clash
 4.0K   │ ┌── hello
 8.0K   ├─┴ dir
 4.0K   │ ┌── hello
 8.0K   ├─┴ dir_substring
  24K ┌─┴ test_dir2
    "
    .trim()
    .into()
}

#[cfg(target_os = "macos")]
fn no_substring_of_names_output() -> String {
    "
   0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_g..
 4.0K   │ ┌── hello
 4.0K   ├─┴ dir
 4.0K   ├── dir_name_clash
 4.0K   │ ┌── hello
 4.0K   ├─┴ dir_substring
  12K ┌─┴ test_dir2
  "
    .trim()
    .into()
}

#[cfg(target_os = "windows")]
fn no_substring_of_names_output() -> String {
    "PRs".into()
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_unicode_directories() {
    initialize();
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd.arg("-c").arg("/tmp/test_dir_unicode").unwrap().stdout;
    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains(&unicode_dir()));
}

#[cfg(target_os = "linux")]
fn unicode_dir() -> String {
    // The way unicode & asian characters are rendered on the terminal should make this line up
    "
   0B   ┌── ラウトは難しいです！.japan│                                 █ │   0%
   0B   ├── 👩.unicode                │                                 █ │   0%
 4.0K ┌─┴ test_dir_unicode            │██████████████████████████████████ │ 100%
    "
    .trim()
    .into()
}

#[cfg(target_os = "macos")]
fn unicode_dir() -> String {
    "
   0B   ┌── ラウトは難しいです！.japan│                                 █ │   0%
   0B   ├── 👩.unicode                │                                 █ │   0%
   0B ┌─┴ test_dir_unicode            │                                 █ │   0%
    "
    .trim()
    .into()
}

#[cfg(target_os = "windows")]
fn unicode_dir() -> String {
    "".into()
}
