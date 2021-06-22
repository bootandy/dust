use assert_cmd::Command;
use std::str;
/**
 * This file contains tests that test a substring of the output using '.contains'
 *
 * These tests should be the same cross platform
 */

// We can at least test the file names are there
#[test]
pub fn test_basic_output() {
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd.arg("tests/test_dir/").unwrap().stdout;
    let output = str::from_utf8(&output).unwrap();

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
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd.arg("-b").arg("tests/test_dir/").unwrap().stdout;
    let output = str::from_utf8(&output).unwrap();
    // If bars are not being shown we don't need to pad the output with spaces
    assert!(output.contains("many"));
    assert!(!output.contains("many    "));
}

#[test]
pub fn test_reverse_flag() {
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-c")
        .arg("-r")
        .arg("tests/test_dir/")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();

    assert!(output.contains(" └─┬ test_dir "));
    assert!(output.contains("  └─┬ many "));
    assert!(output.contains("    ├── hello_file"));
    assert!(output.contains("    └── a_file "));
}

#[test]
pub fn test_d_flag_works() {
    // We should see the top level directory but not the sub dirs / files:
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-d")
        .arg("1")
        .arg("-s")
        .arg("tests/test_dir/")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    assert!(!output.contains("hello_file"));
}

#[test]
pub fn test_d_flag_works_and_still_recurses_down() {
    // We had a bug where running with '-d 1' would stop at the first directory and the code
    // would fail to recurse down
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-d")
        .arg("1")
        .arg("-f")
        .arg("-c")
        .arg("tests/test_dir2/")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains("7 ┌─┴ test_dir2"));
}

// Check against directories and files whos names are substrings of each other
#[test]
pub fn test_ignore_dir() {
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-c")
        .arg("-X")
        .arg("dir_substring")
        .arg("tests/test_dir2")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();
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
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-c")
        .arg("tests/test_dir_hidden_entries")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains(".hidden_file"));
    assert!(output.contains("┌─┴ test_dir_hidden_entries"));

    // Check that adding the '-h' flag causes us to not see hidden files
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-c")
        .arg("-i")
        .arg("tests/test_dir_hidden_entries")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    assert!(!output.contains(".hidden_file"));
    assert!(output.contains("┌── test_dir_hidden_entries"));
}

#[test]
pub fn test_number_of_files() {
    // Check we can see the hidden file normally
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-c")
        .arg("-f")
        .arg("tests/test_dir")
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains("1     ┌── a_file "));
    assert!(output.contains("1     ├── hello_file"));
    assert!(output.contains("3   ┌─┴ many"));
    assert!(output.contains("4 ┌─┴ test_dir"));
}
