use assert_cmd::{Command, cargo_bin_cmd};
use std::ffi::OsStr;
use std::process::Output;
use std::sync::Once;
use std::{io, str};

static INIT: Once = Once::new();
static UNREADABLE_DIR_PATH: &str = "/tmp/unreadable_dir";

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

fn create_unreadable_directory() -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::fs;
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::create_dir_all(UNREADABLE_DIR_PATH)?;
        fs::set_permissions(UNREADABLE_DIR_PATH, Permissions::from_mode(0o0))?;
    }
    Ok(())
}

fn initialize() {
    INIT.call_once(|| {
        copy_test_data("tests/test_dir");
        copy_test_data("tests/test_dir2");
        copy_test_data("tests/test_dir_unicode");

        if let Err(e) = create_unreadable_directory() {
            panic!("Failed to create unreadable directory: {}", e);
        }
    });
}

fn run_cmd<T: AsRef<OsStr>>(command_args: &[T]) -> Output {
    initialize();
    let mut to_run = cargo_bin_cmd!("dust");
    // Hide progress bar
    to_run.arg("-P");
    for p in command_args {
        to_run.arg(p);
    }
    to_run.unwrap()
}

fn exact_stdout_test<T: AsRef<OsStr>>(command_args: &[T], valid_stdout: Vec<String>) {
    let to_run = run_cmd(command_args);

    let stdout_output = str::from_utf8(&to_run.stdout).unwrap().to_owned();
    let will_fail = valid_stdout.iter().any(|i| stdout_output.contains(i));
    if !will_fail {
        eprintln!(
            "output(stdout):\n{}\ndoes not contain any of:\n{}",
            stdout_output,
            valid_stdout.join("\n\n")
        );
    }
    assert!(will_fail);
}

fn exact_stderr_test<T: AsRef<OsStr>>(command_args: &[T], valid_stderr: String) {
    let to_run = run_cmd(command_args);

    let stderr_output = str::from_utf8(&to_run.stderr).unwrap().trim();
    assert_eq!(stderr_output, valid_stderr);
}

// "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_basic() {
    // -c is no color mode - This makes testing much simpler
    exact_stdout_test(&["-c", "-B", "/tmp/test_dir/"], main_output());
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_multi_arg() {
    let command_args = [
        "-c",
        "-B",
        "/tmp/test_dir/many/",
        "/tmp/test_dir",
        "/tmp/test_dir",
    ];
    exact_stdout_test(&command_args, main_output());
}

fn main_output() -> Vec<String> {
    // Some linux currently thought to be Manjaro, Arch
    // Although probably depends on how drive is formatted
    let mac_and_some_linux = r#"
  0B     ┌── a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
4.0Ki     ├── hello_file│████████████████████████████████████████████████ │ 100%
4.0Ki   ┌─┴ many        │████████████████████████████████████████████████ │ 100%
4.0Ki ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%
"#
    .trim()
    .to_string();

    let ubuntu = r#"
   0B     ┌── a_file    │               ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
4.0Ki     ├── hello_file│               ░░░░░░░░░░░░░░░░█████████████████ │  33%
8.0Ki   ┌─┴ many        │               █████████████████████████████████ │  67%
 12Ki ┌─┴ test_dir      │████████████████████████████████████████████████ │ 100%
  "#
    .trim()
    .to_string();

    // 64K block size (e.g. ppc64, some aarch64)
    let large_block = r#"
 0B     ┌── a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
64K     ├── hello_file│██████████████████████████████████████████████████ │ 100%
64K   ┌─┴ many        │██████████████████████████████████████████████████ │ 100%
64K ┌─┴ test_dir      │██████████████████████████████████████████████████ │ 100%
"#
    .trim()
    .to_string();

    vec![mac_and_some_linux, ubuntu, large_block]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_long_paths() {
    let command_args = ["-c", "-p", "-B", "/tmp/test_dir/"];
    exact_stdout_test(&command_args, main_output_long_paths());
}

fn main_output_long_paths() -> Vec<String> {
    let mac_and_some_linux = r#"
   0B     ┌── /tmp/test_dir/many/a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
4.0Ki     ├── /tmp/test_dir/many/hello_file│█████████████████████████████ │ 100%
4.0Ki   ┌─┴ /tmp/test_dir/many             │█████████████████████████████ │ 100%
4.0Ki ┌─┴ /tmp/test_dir                    │█████████████████████████████ │ 100%
"#
    .trim()
    .to_string();
    let ubuntu = r#"
   0B     ┌── /tmp/test_dir/many/a_file    │         ░░░░░░░░░░░░░░░░░░░█ │   0%
4.0Ki     ├── /tmp/test_dir/many/hello_file│         ░░░░░░░░░░██████████ │  33%
8.0Ki   ┌─┴ /tmp/test_dir/many             │         ████████████████████ │  67%
 12Ki ┌─┴ /tmp/test_dir                    │█████████████████████████████ │ 100%
"#
    .trim()
    .to_string();
    let large_block = r#"
 0B     ┌── /tmp/test_dir/many/a_file    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
64K     ├── /tmp/test_dir/many/hello_file│███████████████████████████████ │ 100%
64K   ┌─┴ /tmp/test_dir/many             │███████████████████████████████ │ 100%
64K ┌─┴ /tmp/test_dir                    │███████████████████████████████ │ 100%
"#
    .trim()
    .to_string();

    vec![mac_and_some_linux, ubuntu, large_block]
}

// Check against directories and files whose names are substrings of each other
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_substring_of_names_and_long_names() {
    let command_args = ["-c", "-B", "/tmp/test_dir2"];
    exact_stdout_test(&command_args, no_substring_of_names_output());
}

fn no_substring_of_names_output() -> Vec<String> {
    let ubuntu = "
   0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_goe..
4.0Ki   ├── dir_name_clash
4.0Ki   │ ┌── hello
8.0Ki   ├─┴ dir
4.0Ki   │ ┌── hello
8.0Ki   ├─┴ dir_substring
 24Ki ┌─┴ test_dir2
    "
    .trim()
    .into();

    let mac_and_some_linux = "
   0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_goe..
4.0Ki   │ ┌── hello
4.0Ki   ├─┴ dir
4.0Ki   ├── dir_name_clash
4.0Ki   │ ┌── hello
4.0Ki   ├─┴ dir_substring
 12Ki ┌─┴ test_dir2
  "
    .trim()
    .into();
    let large_block = "
  0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_goes..
 64K   │ ┌── hello
 64K   ├─┴ dir
 64K   ├── dir_name_clash
 64K   │ ┌── hello
 64K   ├─┴ dir_substring
192K ┌─┴ test_dir2
    "
    .trim()
    .into();

    let large_block_alt = "
  0B   ┌── long_dir_name_what_a_very_long_dir_name_what_happens_when_this_goes..
 64K   ├── dir_name_clash
 64K   │ ┌── hello
 64K   ├─┴ dir
 64K   │ ┌── hello
 64K   ├─┴ dir_substring
192K ┌─┴ test_dir2
  "
    .trim()
    .into();

    vec![mac_and_some_linux, ubuntu, large_block, large_block_alt]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_unicode_directories() {
    let command_args = ["-c", "-B", "/tmp/test_dir_unicode"];
    exact_stdout_test(&command_args, unicode_dir());
}

fn unicode_dir() -> Vec<String> {
    // The way unicode & asian characters are rendered on the terminal should make this line up
    let ubuntu = "
   0B   ┌── ラウトは難しいです！.japan│                                 █ │   0%
   0B   ├── 👩.unicode                │                                 █ │   0%
4.0Ki ┌─┴ test_dir_unicode            │██████████████████████████████████ │ 100%
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
    let large_block = "
  0B   ┌── ラウトは難しいです！.japan│                                  █ │   0%
  0B   ├── 👩.unicode                │                                  █ │   0%
 64K ┌─┴ test_dir_unicode            │███████████████████████████████████ │ 100%
    "
    .trim()
    .into();

    vec![mac_and_some_linux, ubuntu, large_block]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_apparent_size() {
    let command_args = ["-c", "-s", "-b", "/tmp/test_dir"];
    exact_stdout_test(&command_args, apparent_size_output());
}

fn apparent_size_output() -> Vec<String> {
    // The apparent directory sizes are too unpredictable and system dependent to try and match
    let two_space_before = r#"
  0B     ┌── a_file
  6B     ├── hello_file
 "#
    .trim()
    .to_string();

    let three_space_before = r#"
   0B     ┌── a_file
   6B     ├── hello_file
 "#
    .trim()
    .to_string();

    vec![two_space_before, three_space_before]
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_permission_normal() {
    let command_args = [UNREADABLE_DIR_PATH];
    let permission_msg =
        r#"Did not have permissions for all directories (add --print-errors to see errors)"#
            .trim()
            .to_string();
    exact_stderr_test(&command_args, permission_msg);
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_permission_flag() {
    // add the flag to CLI
    let command_args = ["--print-errors", UNREADABLE_DIR_PATH];
    let permission_msg = format!(
        "Did not have permissions for directories: {}",
        UNREADABLE_DIR_PATH
    );
    exact_stderr_test(&command_args, permission_msg);
}
