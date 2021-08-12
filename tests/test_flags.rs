use assert_cmd::Command;
use std::ffi::OsStr;
use std::str;

/**
 * This file contains tests that test a substring of the output using '.contains'
 *
 * These tests should be the same cross platform
 */

fn build_command<T: AsRef<OsStr>>(command_args: Vec<T>) -> String {
    let mut a = &mut Command::cargo_bin("dust").unwrap();
    for p in command_args {
        a = a.arg(p);
    }
    str::from_utf8(&a.unwrap().stdout).unwrap().into()
}

// We can at least test the file names are there
#[test]
pub fn test_basic_output() {
    let output = build_command(vec!["tests/test_dir/"]);

    assert!(output.contains(" ┌─┴ "));
    assert!(output.contains("test_dir "));
    assert!(output.contains("  ┌─┴ "));
    assert!(output.contains("many "));
    assert!(output.contains("    ├── "));
    assert!(output.contains("hello_file"));
    assert!(output.contains("     ┌── "));
    assert!(output.contains("a_file "));
}

#[test]
pub fn test_output_no_bars_means_no_excess_spaces() {
    let output = build_command(vec!["-b", "tests/test_dir/"]);
    // If bars are not being shown we don't need to pad the output with spaces
    assert!(output.contains("many"));
    assert!(!output.contains("many    "));
}

#[test]
pub fn test_reverse_flag() {
    let output = build_command(vec!["-r", "-c", "tests/test_dir/"]);
    assert!(output.contains(" └─┬ test_dir "));
    assert!(output.contains("  └─┬ many "));
    assert!(output.contains("    ├── hello_file"));
    assert!(output.contains("    └── a_file "));
}

#[test]
pub fn test_d_flag_works() {
    // We should see the top level directory but not the sub dirs / files:
    let output = build_command(vec!["-d", "1", "tests/test_dir/"]);
    assert!(!output.contains("hello_file"));
}

#[test]
pub fn test_d_flag_works_and_still_recurses_down() {
    // We had a bug where running with '-d 1' would stop at the first directory and the code
    // would fail to recurse down
    let output = build_command(vec!["-d", "1", "-f", "-c", "tests/test_dir2/"]);
    assert!(output.contains("7 ┌─┴ test_dir2"));
}

// Check against directories and files whos names are substrings of each other
#[test]
pub fn test_ignore_dir() {
    let output = build_command(vec!["-c", "-X", "dir_substring", "tests/test_dir2/"]);
    assert!(!output.contains("dir_substring"));
}

#[test]
pub fn test_with_bad_param() {
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let stderr = cmd.arg("-").unwrap().stderr;
    let stderr = str::from_utf8(&stderr).unwrap();
    assert!(stderr.contains("Did not have permissions for all directories"));
}

#[test]
pub fn test_hidden_flag() {
    // Check we can see the hidden file normally
    let output = build_command(vec!["-c", "tests/test_dir_hidden_entries/"]);
    assert!(output.contains(".hidden_file"));
    assert!(output.contains("┌─┴ test_dir_hidden_entries"));

    // Check that adding the '-h' flag causes us to not see hidden files
    let output = build_command(vec!["-c", "-i", "tests/test_dir_hidden_entries/"]);
    assert!(!output.contains(".hidden_file"));
    assert!(output.contains("┌── test_dir_hidden_entries"));
}

#[test]
pub fn test_number_of_files() {
    // Check we can see the hidden file normally
    let output = build_command(vec!["-c", "-f", "tests/test_dir"]);
    assert!(output.contains("1     ┌── a_file "));
    assert!(output.contains("1     ├── hello_file"));
    assert!(output.contains("3   ┌─┴ many"));
    assert!(output.contains("4 ┌─┴ test_dir"));
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_apparent_size() {
    // Check the '-s' Flag gives us byte sizes and that it doesn't round up to a block
    let command_args = vec!["-c", "-s", "/tmp/test_dir"];
    let output = build_command(command_args);

    let apparent_size1 = "6B     ├── hello_file│";
    let apparent_size2 = "0B     ┌── a_file";
    assert!(output.contains(apparent_size1));
    assert!(output.contains(apparent_size2));

    let incorrect_apparent_size = "4.0K     ├── hello_file";
    assert!(!output.contains(incorrect_apparent_size));
}

#[test]
pub fn test_show_files_by_type() {
    // Check we can list files by type
    let output = build_command(vec!["-c", "-t", "tests"]);
    assert!(output.contains(" .unicode"));
    assert!(output.contains(" .japan"));
    assert!(output.contains(" .rs"));
    assert!(output.contains(" (no extension)"));
    assert!(output.contains("┌─┴ (total)"));
}

#[test]
pub fn test_show_files_by_specific_type() {
    // Check we can see '.rs' files in the tests directory
    let output = build_command(vec!["-c", "-y", "rs", "tests"]);
    assert!(output.contains(" ┌── tests│"));
    assert!(!output.contains("0B ┌── tests│"));

    // Check there are no '.bad_type' files in the tests directory
    let output = build_command(vec!["-c", "-y", "bad_type", "tests"]);
    assert!(output.contains("0B ┌── tests│"));
}
