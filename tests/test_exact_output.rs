use assert_cmd::Command;
use std::ffi::OsStr;
use std::str;
use std::sync::Once;

static INIT: Once = Once::new();

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
    // First remove the existing directory - just in case it is there and has incorrect data
    let last_slash = dir.rfind('/').unwrap();
    let last_part_of_dir = dir.chars().skip(last_slash).collect::<String>();
    let _ = Command::new("rm")
        .arg("-rf")
        .arg("/tmp/".to_owned() + &*last_part_of_dir)
        .ok();

    let _ = Command::new("cp")
        .arg("-r")
        .arg(dir)
        .arg("/tmp/")
        .ok()
        .map_err(|err| eprintln!("Error copying directory for test setup\n{:?}", err));
}

fn initialize() {
    INIT.call_once(|| {
        copy_test_data("tests/test_dir");
        copy_test_data("tests/test_dir2");
        copy_test_data("tests/test_dir_unicode");
    });
}

fn exact_output_test<T: AsRef<OsStr>>(
    command_args: Vec<T>,
    valid_outputs: Vec<String>,
    error_outputs: Vec<String>,
) {
    initialize();

    let mut a = &mut Command::cargo_bin("dust").unwrap();

    for p in command_args {
        a = a.arg(p);
    }

    if !valid_outputs.is_empty() {
        let stdout_output = str::from_utf8(&a.unwrap().stdout).unwrap().to_owned();
        let will_fail = valid_outputs.iter().any(|i| stdout_output.contains(i));
        if !will_fail {
            eprintln!(
                "output(stdout):\n{}\ndoes not contain any of:\n{}",
                stdout_output,
                valid_outputs.join("\n\n")
            );
        }
        assert!(will_fail);
    }

    // check stderr
    if !error_outputs.is_empty() {
        let stderr_output = str::from_utf8(&a.unwrap().stderr).unwrap().to_owned();
        let will_fail = error_outputs.iter().any(|i| stderr_output.contains(i));
        if !will_fail {
            eprintln!(
                "output(stderr):\n{}\ndoes not contain any of:\n{}",
                stderr_output,
                valid_outputs.join("\n\n")
            );
        }
        assert!(will_fail);
    }
}

// "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_basic() {
    // -c is no color mode - This makes testing much simpler
    exact_output_test(vec!["-c", "-B", "/tmp/test_dir/"], main_output(), vec![]);
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_multi_arg() {
    let command_args = vec![
        "-c",
        "-B",
        "/tmp/test_dir/many/",
        "/tmp/test_dir",
        "/tmp/test_dir",
    ];
    exact_output_test(command_args, main_output(), vec![]);
}

fn main_output() -> Vec<String> {
    // Some linux currently thought to be Manjaro, Arch
    // Although probably depends on how drive is formatted
    let mac_and_some_linux = r#"
  0B     ┌── a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
4.0K     ├── hello_file│█████████████████████████████████████████████████ │ 100%
4.0K   ┌─┴ many        │█████████████████████████████████████████████████ │ 100%
4.0K ┌─┴ test_dir      │█████████████████████████████████████████████████ │ 100%
"#
    .trim()
    .to_string();

    let ubuntu = r#"
  0B     ┌── a_file    │                ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
4.0K     ├── hello_file│                ░░░░░░░░░░░░░░░░█████████████████ │  33%
8.0K   ┌─┴ many        │                █████████████████████████████████ │  67%
 12K ┌─┴ test_dir      │█████████████████████████████████████████████████ │ 100%
  "#
    .trim()
    .to_string();

    vec![mac_and_some_linux, ubuntu]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_long_paths() {
    let command_args = vec!["-c", "-p", "-B", "/tmp/test_dir/"];
    exact_output_test(command_args, main_output_long_paths(), vec![]);
}

fn main_output_long_paths() -> Vec<String> {
    let mac_and_some_linux = r#"
  0B     ┌── /tmp/test_dir/many/a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
4.0K     ├── /tmp/test_dir/many/hello_file│██████████████████████████████ │ 100%
4.0K   ┌─┴ /tmp/test_dir/many             │██████████████████████████████ │ 100%
4.0K ┌─┴ /tmp/test_dir                    │██████████████████████████████ │ 100%
"#
    .trim()
    .to_string();
    let ubuntu = r#"
  0B     ┌── /tmp/test_dir/many/a_file    │         ░░░░░░░░░░░░░░░░░░░░█ │   0%
4.0K     ├── /tmp/test_dir/many/hello_file│         ░░░░░░░░░░███████████ │  33%
8.0K   ┌─┴ /tmp/test_dir/many             │         █████████████████████ │  67%
 12K ┌─┴ /tmp/test_dir                    │██████████████████████████████ │ 100%
"#
    .trim()
    .to_string();
    vec![mac_and_some_linux, ubuntu]
}

// Check against directories and files whose names are substrings of each other
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_substring_of_names_and_long_names() {
    let command_args = vec!["-c", "-B", "/tmp/test_dir2"];
    exact_output_test(command_args, no_substring_of_names_output(), vec![]);
}

fn no_substring_of_names_output() -> Vec<String> {
    let ubuntu = "
  0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_goes..
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
  0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_goes..
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
    let command_args = vec!["-c", "-B", "/tmp/test_dir_unicode"];
    exact_output_test(command_args, unicode_dir(), vec![]);
}

fn unicode_dir() -> Vec<String> {
    // The way unicode & asian characters are rendered on the terminal should make this line up
    let ubuntu = "
  0B   ┌── ラウトは難しいです！.japan│                                  █ │   0%
  0B   ├── 👩.unicode                │                                  █ │   0%
4.0K ┌─┴ test_dir_unicode            │███████████████████████████████████ │ 100%
    "
    .trim()
    .into();

    let mac_and_some_linux = "
0B   ┌── ラウトは難しいです！.japan│                                    █ │   0%
0B   ├── 👩.unicode                │                                    █ │   0%
0B ┌─┴ test_dir_unicode            │                                    █ │   0%
    "
    .trim()
    .into();
    vec![mac_and_some_linux, ubuntu]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_apparent_size() {
    let command_args = vec!["-c", "-s", "-b", "/tmp/test_dir"];
    exact_output_test(command_args, apparent_size_output(), vec![]);
}

fn apparent_size_output() -> Vec<String> {
    // The apparent directory sizes are too unpredictable and system dependent to try and match
    let one_space_before = r#"
 0B     ┌── a_file
 6B     ├── hello_file
 "#
    .trim()
    .to_string();

    let two_space_before = r#"
  0B     ┌── a_file
  6B     ├── hello_file
 "#
    .trim()
    .to_string();

    vec![one_space_before, two_space_before]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_permission() {
    Command::new("sh")
        .arg("-c")
        .arg("mkdir -p /tmp/unreadable_folder && chmod 000 /tmp/unreadable_folder")
        .output()
        .unwrap();
    let command_args = vec!["/tmp/unreadable_folder"];
    let permission_msg = r#"Did not have permissions"#.trim().to_string();
    exact_output_test(command_args, vec![], vec![permission_msg]);

    Command::new("sh")
        .arg("-c")
        .arg("chmod 555 /tmp/unreadable_folder")
        .output()
        .unwrap();
}
