use assert_cmd::Command;
use std::ffi::OsStr;
use std::str;

/**
 * This file contains tests that test a substring of the output using '.contains'
 *
 * These tests should be the same cross platform
 */

fn build_command<T: AsRef<OsStr>>(command_args: Vec<T>) -> String {
    let mut cmd = &mut Command::cargo_bin("dust").unwrap();
    for p in command_args {
        cmd = cmd.arg(p);
    }
    let finished = &cmd.unwrap();
    let stderr = str::from_utf8(&finished.stderr).unwrap();
    assert_eq!(stderr, "");

    str::from_utf8(&finished.stdout).unwrap().into()
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
pub fn test_threads_flag_works() {
    let output = build_command(vec!["-T", "1", "tests/test_dir/"]);
    assert!(output.contains("hello_file"));
}

#[test]
pub fn test_d_flag_works_and_still_recurses_down() {
    // We had a bug where running with '-d 1' would stop at the first directory and the code
    // would fail to recurse down
    let output = build_command(vec!["-d", "1", "-f", "-c", "tests/test_dir2/"]);
    assert!(output.contains("1   ┌── dir"));
    assert!(output.contains("4 ┌─┴ test_dir2"));
}

// Check against directories and files whose names are substrings of each other
#[test]
pub fn test_ignore_dir() {
    let output = build_command(vec!["-c", "-X", "dir_substring", "tests/test_dir2/"]);
    assert!(!output.contains("dir_substring"));
}

#[test]
pub fn test_ignore_all_in_file() {
    let output = build_command(vec![
        "-c",
        "-I",
        "tests/test_dir_hidden_entries/.hidden_file",
        "tests/test_dir_hidden_entries/",
    ]);
    assert!(output.contains(" test_dir_hidden_entries"));
    assert!(!output.contains(".secret"));
}

#[test]
pub fn test_with_bad_param() {
    let mut cmd = Command::cargo_bin("dust").unwrap();
    let result = cmd.arg("bad_place").unwrap();
    let stderr = str::from_utf8(&result.stderr).unwrap();
    assert!(stderr.contains("No such file or directory"));
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
    assert!(output.contains("2   ┌─┴ many"));
    assert!(output.contains("2 ┌─┴ test_dir"));
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
#[cfg(target_family = "unix")]
pub fn test_show_files_only() {
    let output = build_command(vec!["-c", "-F", "tests/test_dir"]);
    assert!(output.contains("a_file"));
    assert!(output.contains("hello_file"));
    assert!(!output.contains("many"));
}

#[test]
pub fn test_output_skip_total() {
    let output = build_command(vec![
        "--skip-total",
        "tests/test_dir/many/hello_file",
        "tests/test_dir/many/a_file",
    ]);
    assert!(output.contains("hello_file"));
    assert!(!output.contains("(total)"));
}

#[test]
pub fn test_output_screen_reader() {
    let output = build_command(vec!["--screen-reader", "-c", "tests/test_dir/"]);
    println!("{}", output);
    assert!(output.contains("test_dir   0"));
    assert!(output.contains("many       1"));
    assert!(output.contains("hello_file 2"));
    assert!(output.contains("a_file     2"));

    // Verify no 'symbols' reported by screen reader
    assert!(!output.contains('│'));

    for block in ['█', '▓', '▒', '░'] {
        assert!(!output.contains(block));
    }
}

#[test]
pub fn test_show_files_by_regex_match_lots() {
    // Check we can see '.rs' files in the tests directory
    let output = build_command(vec!["-c", "-e", "\\.rs$", "tests"]);
    assert!(output.contains(" ┌─┴ tests"));
    assert!(!output.contains("0B ┌── tests"));
    assert!(!output.contains("0B ┌─┴ tests"));
}

#[test]
pub fn test_show_files_by_regex_match_nothing() {
    // Check there are no files named: '.match_nothing' in the tests directory
    let output = build_command(vec!["-c", "-e", "match_nothing$", "tests"]);
    assert!(output.contains("0B ┌── tests"));
}

#[test]
pub fn test_show_files_by_regex_match_multiple() {
    let output = build_command(vec![
        "-c",
        "-e",
        "test_dir_hidden",
        "-e",
        "test_dir2",
        "-n",
        "100",
        "tests",
    ]);
    assert!(output.contains("test_dir2"));
    assert!(output.contains("test_dir_hidden"));
    assert!(!output.contains("many")); // We do not find the 'many' folder in the 'test_dir' folder
}

#[test]
pub fn test_show_files_by_invert_regex() {
    let output = build_command(vec!["-c", "-f", "-v", "e", "tests/test_dir2"]);
    // There are 0 files without 'e' in the name
    assert!(output.contains("0 ┌── test_dir2"));

    let output = build_command(vec!["-c", "-f", "-v", "a", "tests/test_dir2"]);
    // There are 2 files without 'a' in the name
    assert!(output.contains("2 ┌─┴ test_dir2"));

    // There are 4 files in the test_dir2 hierarchy
    let output = build_command(vec!["-c", "-f", "-v", "match_nothing$", "tests/test_dir2"]);
    assert!(output.contains("4 ┌─┴ test_dir2"));
}

#[test]
pub fn test_show_files_by_invert_regex_match_multiple() {
    // We ignore test_dir2 & test_dir_unicode, leaving the test_dir folder
    // which has the 'many' folder inside
    let output = build_command(vec![
        "-c",
        "-v",
        "test_dir2",
        "-v",
        "test_dir_unicode",
        "-n",
        "100",
        "tests",
    ]);
    assert!(!output.contains("test_dir2"));
    assert!(!output.contains("test_dir_unicode"));
    assert!(output.contains("many"));
}

#[test]
pub fn test_no_color() {
    let output = build_command(vec!["-c"]);
    // Red is 31
    assert!(!output.contains("\x1B[31m"));
    assert!(!output.contains("\x1B[0m"));
}

#[test]
pub fn test_force_color() {
    let output = build_command(vec!["-C"]);
    // Red is 31
    assert!(output.contains("\x1B[31m"));
    assert!(output.contains("\x1B[0m"));
}

#[test]
pub fn test_collapse() {
    let output = build_command(vec!["--collapse", "many", "tests/test_dir/"]);
    assert!(output.contains("many"));
    assert!(!output.contains("hello_file"));
}

#[test]
pub fn test_handle_duplicate_names() {
    // Check that even if we run on a multiple directories with the same name
    // we still show the distinct parent dir in the output
    let output = build_command(vec![
        "tests/test_dir_matching/dave/dup_name",
        "tests/test_dir_matching/andy/dup_name",
        "ci",
    ]);
    assert!(output.contains("andy"));
    assert!(output.contains("dave"));
    assert!(output.contains("ci"));
    assert!(output.contains("dup_name"));
    assert!(!output.contains("test_dir_matching"));
}
