use assert_cmd::Command;
use std::ffi::OsStr;
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

fn initialize() {
    INIT.call_once(|| {
        copy_test_data("tests/test_dir");
        copy_test_data("tests/test_dir2");
        copy_test_data("tests/test_dir_unicode");
    });
}

fn run_dust_with<T: AsRef<OsStr>>(params: Vec<T>) -> String {
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let mut a = &mut cmd;
    for p in params {
        a = a.arg(p);
    }
    str::from_utf8(&a.unwrap().stdout).unwrap().into()
}

fn check_dust_output(output: String, func: fn() -> Vec<String>) {
    assert!(func()
        .iter()
        .fold(false, |sum, i| sum || output.contains(i)));
}

// "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_basic() {
    // -c is no color mode - This makes testing much simpler
    initialize();
    let output = run_dust_with(vec!["-c", "/tmp/test_dir/"]);
    check_dust_output(output, main_output);
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_multi_arg() {
    initialize();
    let output = run_dust_with(vec![
        "-c",
        "/tmp/test_dir/many/",
        "/tmp/test_dir",
        "/tmp/test_dir",
    ]);
    check_dust_output(output, main_output);
}

fn main_output() -> Vec<String> {
    // Some linux currently thought to be Manjaro, Arch
    // Although probably depends on how drive is formatted
    let mac_and_some_linux = r#"
   0B     ┌── a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── hello_file│████████████████████████████████████████████████ │ 100%
 4.0K   ┌─┴ many        │████████████████████████████████████████████████ │ 100%
 4.0K ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%
 "#
    .trim()
    .to_string();

    let ubuntu = r#"
   0B     ┌── a_file    │               ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── hello_file│               ░░░░░░░░░░░░░░░░█████████████████ │  33%
 8.0K   ┌─┴ many        │               █████████████████████████████████ │  67%
  12K ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%
  "#
    .trim()
    .to_string();

    vec![mac_and_some_linux, ubuntu]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_long_paths() {
    initialize();
    let output = run_dust_with(vec!["-c", "-p", "/tmp/test_dir/"]);
    check_dust_output(output, main_output_long_paths);
}

fn main_output_long_paths() -> Vec<String> {
    let mac_and_some_linux = r#"
   0B     ┌── /tmp/test_dir/many/a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── /tmp/test_dir/many/hello_file│█████████████████████████████ │ 100%
 4.0K   ┌─┴ /tmp/test_dir/many             │█████████████████████████████ │ 100%
 4.0K ┌─┴ /tmp/test_dir                    │█████████████████████████████ │ 100%
"#
    .trim()
    .to_string();
    let ubuntu = r#"
   0B     ┌── /tmp/test_dir/many/a_file    │         ░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── /tmp/test_dir/many/hello_file│         ░░░░░░░░░░██████████ │  33%
 8.0K   ┌─┴ /tmp/test_dir/many             │         ████████████████████ │  67%
  12K ┌─┴ /tmp/test_dir                    │█████████████████████████████ │ 100%
"#
    .trim()
    .to_string();
    vec![mac_and_some_linux, ubuntu]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_apparent_size() {
    initialize();
    let output = run_dust_with(vec!["-c", "-s", "/tmp/test_dir"]);
    check_dust_output(output, output_apparent_size);
}

fn output_apparent_size() -> Vec<String> {
    // The directory sizes vary a lot based on what the underlying filesystem is
    // so different distros give different results. Really we should be checking that
    // the standard '4.0K' isn't there
    let apparent_size = "6B     ├── hello_file│".into();
    vec![apparent_size]
}

// Check against directories and files whos names are substrings of each other
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_substring_of_names_and_long_names() {
    initialize();
    let output = run_dust_with(vec!["-c", "/tmp/test_dir2"]);
    check_dust_output(output, no_substring_of_names_output);
}

fn no_substring_of_names_output() -> Vec<String> {
    let ubuntu = "
   0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_g..
 4.0K   ├── dir_name_clash
 4.0K   │ ┌── hello
 8.0K   ├─┴ dir
 4.0K   │ ┌── hello
 8.0K   ├─┴ dir_substring
  24K ┌─┴ test_dir2
    "
    .trim()
    .into();

    let mac_and_some_linux = "
   0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_g..
 4.0K   │ ┌── hello
 4.0K   ├─┴ dir
 4.0K   ├── dir_name_clash
 4.0K   │ ┌── hello
 4.0K   ├─┴ dir_substring
  12K ┌─┴ test_dir2
  "
    .trim()
    .into();
    vec![mac_and_some_linux, ubuntu]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_unicode_directories() {
    initialize();
    let output = run_dust_with(vec!["-c", "/tmp/test_dir_unicode"]);
    check_dust_output(output, unicode_dir);
}

fn unicode_dir() -> Vec<String> {
    // The way unicode & asian characters are rendered on the terminal should make this line up
    let ubuntu = "
   0B   ┌── ラウトは難しいです！.japan│                                 █ │   0%
   0B   ├── 👩.unicode                │                                 █ │   0%
 4.0K ┌─┴ test_dir_unicode            │██████████████████████████████████ │ 100%
    "
    .trim()
    .into();

    let mac_and_some_linux = "
   0B   ┌── ラウトは難しいです！.japan│                                 █ │   0%
   0B   ├── 👩.unicode                │                                 █ │   0%
   0B ┌─┴ test_dir_unicode            │                                 █ │   0%
    "
    .trim()
    .into();
    vec![mac_and_some_linux, ubuntu]
}
