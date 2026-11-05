use assert_cmd::cargo_bin_cmd;
use chrono::{Local, TimeZone};
use std::ffi::OsStr;
use std::fs::{FileTimes, OpenOptions};
use std::str;

/**
 * This file contains tests that test a substring of the output using '.contains'
 *
 * These tests should be the same cross platform
 */

fn build_command<T: AsRef<OsStr>>(command_args: Vec<T>) -> String {
    let mut cmd = cargo_bin_cmd!("dust");

    // Hide progress bar
    cmd.arg("-P");

    for p in command_args {
        cmd.arg(p);
    }
    let finished = &cmd.unwrap();
    assert_eq!(str::from_utf8(&finished.stderr).unwrap(), "");
    str::from_utf8(&finished.stdout).unwrap().into()
}

#[test]
fn test_filetime_output_uses_unix_timestamp() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(temp_dir.path().join("recent.txt"), b"recent").unwrap();

    let mut cmd = cargo_bin_cmd!("dust");
    let output = cmd
        .arg("-P")
        .arg("-c")
        .arg("--filetime")
        .arg("modified")
        .arg(temp_dir.path())
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        str::from_utf8(&output.stderr).unwrap()
    );
}

#[test]
fn test_mtime_filter_uses_unix_timestamp() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("yesterday.txt");
    std::fs::write(&file_path, b"yesterday").unwrap();

    let yesterday = Local::now().date_naive().pred_opt().unwrap();
    let yesterday_noon = Local
        .from_local_datetime(&yesterday.and_hms_opt(12, 0, 0).unwrap())
        .single()
        .unwrap();
    let file = OpenOptions::new().write(true).open(&file_path).unwrap();
    file.set_times(FileTimes::new().set_modified(yesterday_noon.into()))
        .unwrap();

    let mut cmd = cargo_bin_cmd!("dust");
    let output = cmd
        .arg("-P")
        .arg("-c")
        .arg("--mtime")
        .arg("0")
        .arg(temp_dir.path())
        .unwrap();

    assert!(output.status.success());
    assert!(
        str::from_utf8(&output.stdout)
            .unwrap()
            .contains("yesterday.txt")
    );
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
pub fn test_d0_works_on_multiple() {
    // We should see the top level directory but not the sub dirs / files:
    let output = build_command(vec!["-d", "0", "tests/test_dir/", "tests/test_dir2"]);
    assert!(output.contains("test_dir "));
    assert!(output.contains("test_dir2"));
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
pub fn test_files_from_flag_file() {
    let output = build_command(vec![
        "--files-from",
        "tests/test_dir_files_from/files_from.txt",
    ]);
    assert!(output.contains("a_file"));
    assert!(output.contains("hello_file"));
}

#[test]
pub fn test_files0_from_flag_file() {
    let output = build_command(vec![
        "--files0-from",
        "tests/test_dir_files_from/files0_from.txt",
    ]);
    assert!(output.contains("a_file"));
    assert!(output.contains("hello_file"));
}

#[test]
pub fn test_files_from_flag_stdin() {
    let mut cmd = cargo_bin_cmd!("dust");
    cmd.arg("-P").arg("--files-from").arg("-");
    let input = b"tests/test_dir_files_from/a_file\ntests/test_dir_files_from/hello_file\n";
    cmd.write_stdin(input.as_ref());
    let finished = &cmd.unwrap();
    let stderr = std::str::from_utf8(&finished.stderr).unwrap();
    assert_eq!(stderr, "");
    let output = std::str::from_utf8(&finished.stdout).unwrap();
    assert!(output.contains("a_file"));
    assert!(output.contains("hello_file"));
}

#[test]
pub fn test_files_from_ignores_empty_lines() {
    let mut cmd = cargo_bin_cmd!("dust");
    cmd.arg("-P").arg("--files-from").arg("-");
    let input = b"tests/test_dir_files_from/a_file\n\ntests/test_dir_files_from/hello_file\n";
    cmd.write_stdin(input.as_ref());
    let finished = &cmd.unwrap();
    let stderr = str::from_utf8(&finished.stderr).unwrap();
    assert_eq!(stderr, "");
    let output = str::from_utf8(&finished.stdout).unwrap();
    assert!(output.contains("a_file"));
    assert!(output.contains("hello_file"));
}

#[test]
pub fn test_cli_paths_ignore_empty_arguments() {
    let output = build_command(vec![
        "-b",
        "-c",
        "-d",
        "0",
        "tests/test_dir_files_from/a_file",
        "",
        "tests/test_dir_files_from/hello_file",
    ]);
    assert!(output.contains("a_file"));
    assert!(output.contains("hello_file"));
}

#[test]
pub fn test_files0_from_flag_stdin() {
    let mut cmd = cargo_bin_cmd!("dust");
    cmd.arg("-P").arg("--files0-from").arg("-");
    let input = b"tests/test_dir_files_from/a_file\0tests/test_dir_files_from/hello_file\0";
    cmd.write_stdin(input.as_ref());
    let finished = &cmd.unwrap();
    let stderr = std::str::from_utf8(&finished.stderr).unwrap();
    assert_eq!(stderr, "");
    let output = std::str::from_utf8(&finished.stdout).unwrap();
    assert!(output.contains("a_file"));
    assert!(output.contains("hello_file"));
}

#[test]
pub fn test_with_bad_param() {
    let mut cmd = cargo_bin_cmd!("dust");
    cmd.arg("-P").arg("bad_place");
    let output_error = cmd.unwrap_err();
    let result = output_error.as_output().unwrap();
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

// Unix only: on windows the metadata time is a FILETIME (100ns ticks since
// 1601), not a unix epoch, so `-m` panics in get_pretty_file_modified_time.
// That is pre-existing and unrelated to this fix.
#[cfg(target_family = "unix")]
#[test]
pub fn test_show_files_by_type_with_filetime() {
    // When grouping by file type and showing file times, the 'size' of a group is
    // a timestamp: it must be the newest file's time, not the sum of the times.
    use std::fs::File;
    use std::time::{Duration, UNIX_EPOCH};

    let dir = tempfile::Builder::new().tempdir().unwrap();

    // Midday UTC, so the year is the same in every timezone
    for epoch_seconds in [1593604800, 1625140800, 1656676800] {
        let file = File::create(dir.path().join(format!("{epoch_seconds}.log"))).unwrap();
        file.set_modified(UNIX_EPOCH + Duration::from_secs(epoch_seconds))
            .unwrap();
    }

    let output = build_command(vec!["-c", "-t", "-m", "m", dir.path().to_str().unwrap()]);

    // 1656676800 is 2022-07-01, the newest of the three
    assert!(output.contains("2022-07-0"), "{output}");
    assert!(!output.contains("2020-07-0"), "{output}");
    assert!(!output.contains("2021-07-0"), "{output}");
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

#[cfg(target_os = "macos")]
#[test]
pub fn test_shared_blocks_counts_clones_once() {
    use std::process::Command;

    let dir = tempfile::Builder::new().tempdir().unwrap();
    let original = dir.path().join("original");
    std::fs::write(&original, vec![7u8; 100_000]).unwrap();
    // `cp -c` makes an APFS clone that shares the original's blocks
    let status = Command::new("cp")
        .arg("-c")
        .arg(&original)
        .arg(dir.path().join("clone"))
        .status()
        .unwrap();
    assert!(status.success());

    // With -d 0 the only line is the total, and its first word the size
    let total = |args: Vec<&str>| -> u64 {
        let output = build_command([vec!["-c", "-b", "-o", "b", "-d", "0"], args].concat());
        output
            .split_whitespace()
            .next()
            .unwrap()
            .trim_end_matches('B')
            .parse()
            .unwrap()
    };
    let path = dir.path().to_str().unwrap();
    let file_size = total(vec![original.to_str().unwrap()]);
    let counted_twice = total(vec![path]);
    let counted_once = total(vec!["--shared-blocks", path]);

    assert_eq!(counted_twice - counted_once, file_size);
}

// A hole maps to device offset -1; reading it as a real extent overflowed.
// Only files that may share blocks get their extents read, so clone it.
#[cfg(all(target_os = "macos", feature = "exact-shared-blocks"))]
#[test]
pub fn test_shared_blocks_cloned_sparse_file() {
    use std::io::{Seek, SeekFrom, Write};
    use std::process::Command;

    let dir = tempfile::Builder::new().tempdir().unwrap();
    let sparse = dir.path().join("sparse");
    let mut file = std::fs::File::create(&sparse).unwrap();
    file.seek(SeekFrom::Start(50 * 1024 * 1024)).unwrap();
    file.write_all(&[7u8; 4096]).unwrap();
    drop(file);
    let status = Command::new("cp")
        .arg("-c")
        .arg(&sparse)
        .arg(dir.path().join("clone"))
        .status()
        .unwrap();
    assert!(status.success());

    let total = |args: Vec<&str>| -> u64 {
        let output = build_command([vec!["-c", "-b", "-o", "b", "-d", "0"], args].concat());
        output
            .split_whitespace()
            .next()
            .unwrap()
            .trim_end_matches('B')
            .parse()
            .unwrap()
    };
    let path = dir.path().to_str().unwrap();
    let file_size = total(vec![sparse.to_str().unwrap()]);
    let counted_twice = total(vec![path]);
    let counted_once = total(vec!["--shared-blocks", path]);

    assert_eq!(counted_twice - counted_once, file_size);
}

// Needs a btrfs or XFS mount: run with
// `DUST_REFLINK_DIR=<dir on that mount> cargo test -- --ignored shared_blocks`
#[cfg(target_os = "linux")]
#[test]
#[ignore = "needs a reflink-capable filesystem in DUST_REFLINK_DIR"]
pub fn test_shared_blocks_counts_reflinks_once() {
    use std::io::{Seek, SeekFrom, Write};
    use std::process::Command;

    let root = std::env::var("DUST_REFLINK_DIR").expect("DUST_REFLINK_DIR is not set");
    let dir = tempfile::Builder::new().tempdir_in(root).unwrap();
    let original = dir.path().join("original");
    let mut file = std::fs::File::create(&original).unwrap();
    file.write_all(&[7u8; 1024 * 1024]).unwrap();
    // Unwritten data has no physical address yet (delayed allocation)
    file.sync_all().unwrap();
    for clone in ["clone", "partial"] {
        let status = Command::new("cp")
            .arg("--reflink=always")
            .arg(&original)
            .arg(dir.path().join(clone))
            .status()
            .unwrap();
        assert!(status.success());
    }
    // Rewrite one 4 KiB block of `partial`, which unshares just that block
    let mut partial = std::fs::OpenOptions::new()
        .write(true)
        .open(dir.path().join("partial"))
        .unwrap();
    partial.seek(SeekFrom::Start(64 * 1024)).unwrap();
    partial.write_all(&[9u8; 4096]).unwrap();
    partial.sync_all().unwrap();

    let total = |args: Vec<&str>| -> u64 {
        let output = build_command([vec!["-c", "-b", "-o", "b", "-d", "0"], args].concat());
        output
            .split_whitespace()
            .next()
            .unwrap()
            .trim_end_matches('B')
            .parse()
            .unwrap()
    };
    let path = dir.path().to_str().unwrap();
    let file_size = total(vec![original.to_str().unwrap()]);
    let counted_thrice = total(vec![path]);
    let counted_once = total(vec!["--shared-blocks", path]);

    // The clone is free; the partial clone costs only its rewritten block
    assert_eq!(counted_thrice - counted_once, 2 * file_size - 4096);
}
