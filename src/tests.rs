use super::*;

use crate::display::DisplayData;
use display::format_string;
use std::fs::File;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;
use tempfile::TempDir;

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main() {
    assert_cli::Assert::main_binary()
        .with_args(&["src/test_dir"])
        .stdout()
        .is(main_output(true).as_str())
        .unwrap();
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_long_paths() {
    assert_cli::Assert::main_binary()
        .with_args(&["-p", "src/test_dir"])
        .stdout()
        .is(main_output(false).as_str())
        .unwrap();
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_multi_arg() {
    assert_cli::Assert::main_binary()
        .with_args(&["src/test_dir/many/", "src/test_dir/", "src/test_dir"])
        .stdout()
        .is(main_output(true).as_str())
        .unwrap();
}

#[cfg(target_os = "macos")]
fn main_output(short_paths: bool) -> String {
    let d = DisplayData {
        short_paths,
        is_reversed: false,
        colors_on: true,
        terminal_size: None,
    };
    format!(
        "{}
{}
{}
{}",
        format_string("src/test_dir", true, &d, " 4.0K", "─┬"),
        format_string("src/test_dir/many", true, &d, " 4.0K", " └─┬",),
        format_string("src/test_dir/many/hello_file", true, &d, " 4.0K", "   ├──",),
        format_string("src/test_dir/many/a_file", false, &d, "   0B", "   └──",),
    )
}

#[cfg(target_os = "linux")]
fn main_output(short_paths: bool) -> String {
    let d = DisplayData {
        short_paths,
        is_reversed: false,
        colors_on: true,
        terminal_size: None,
    };
    format!(
        "{}
{}
{}
{}",
        format_string("src/test_dir", true, &d, "  12K", "100%", "─┬"),
        format_string("src/test_dir/many", true, &d, " 8.0K", "67%", " └─┬",),
        format_string(
            "src/test_dir/many/hello_file",
            true,
            &d,
            " 4.0K",
            "33%",
            "   ├──",
        ),
        format_string(
            "src/test_dir/many/a_file",
            false,
            &d,
            "   0B",
            "0%",
            "   └──",
        ),
    )
}

#[cfg(target_os = "windows")]
fn main_output(short_paths: bool) -> String {
    let d = DisplayData {
        short_paths,
        is_reversed: false,
        colors_on: true,
        terminal_size: None,
    };
    format!(
        "{}
{}
{}
{}",
        format_string("src/test_dir", true, &d, "   6B", "─┬"),
        format_string("src/test_dir\\many", true, &d, "   6B", " └─┬",),
        format_string(
            "src/test_dir\\many\\hello_file",
            true,
            &d,
            "   6B",
            "   ├──",
        ),
        format_string("src/test_dir\\many\\a_file", false, &d, "   0B", "   └──",),
    )
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_no_color_flag() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "src/test_dir/"])
        .stdout()
        .is(no_color_flag_output().as_str())
        .unwrap();
}

#[cfg(target_os = "macos")]
fn no_color_flag_output() -> String {
    "
 4.0K ─┬ test_dir
 4.0K  └─┬ many
 4.0K    ├── hello_file
   0B    └── a_file
    "
    .to_string()
}

#[cfg(target_os = "linux")]
fn no_color_flag_output() -> String {
    "
  12K ─┬ test_dir
 8.0K  └─┬ many
 4.0K    ├── hello_file
   0B    └── a_file
    "
    .to_string()
}

#[cfg(target_os = "windows")]
fn no_color_flag_output() -> String {
    "
   6B ─┬ test_dir
   6B  └─┬ many
   6B    ├── hello_file
   0B    └── a_file
    "
    .to_string()
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_apparent_size() {
    let d = DisplayData {
        short_paths: true,
        is_reversed: false,
        colors_on: true,
        terminal_size: None,
    };
    let r = format!(
        "{}",
        format_string(
            "src/test_dir/many/hello_file",
            true,
            &d,
            "   6B",
            "100%",
            "   ├──",
        ),
    );

    assert_cli::Assert::main_binary()
        .with_args(&["-s", "src/test_dir"])
        .stdout()
        .contains(r.as_str())
        .unwrap();
}

#[test]
pub fn test_reverse_flag() {
    // variable names the same length make the output easier to read
    let a = "    ┌── a_file";
    let b = "    ├── hello_file";
    let c = "  ┌─┴ many";
    let d = " ─┴ test_dir";

    assert_cli::Assert::main_binary()
        .with_args(&["-r", "src/test_dir"])
        .stdout()
        .contains(a)
        .stdout()
        .contains(b)
        .stdout()
        .contains(c)
        .stdout()
        .contains(d)
        .unwrap();
}

#[test]
pub fn test_d_flag_works() {
    // We should see the top level directory but not the sub dirs / files:
    assert_cli::Assert::main_binary()
        .with_args(&["-d", "1", "-s", "src/test_dir"])
        .stdout()
        .doesnt_contain("hello_file")
        .unwrap();
}

fn build_temp_file(dir: &TempDir) -> PathBuf {
    let file_path = dir.path().join("notes.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "I am a temp file").unwrap();
    file_path
}

// fix! [rivy; 2020-01-22] possible on "windows"?; `ln` is not usually an available command; creation of symbolic links requires special enhanced permissions
//  ... ref: <https://superuser.com/questions/343074/directory-junction-vs-directory-symbolic-link> @@ <https://archive.is/gpTLE>
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_soft_sym_link() {
    let dir = Builder::new().tempdir().unwrap();
    let file = build_temp_file(&dir);
    let dir_s = dir.path().to_str().unwrap();
    let file_path_s = file.to_str().unwrap();

    let link_name = dir.path().join("the_link");
    let link_name_s = link_name.to_str().unwrap();
    let c = Command::new("ln")
        .arg("-s")
        .arg(file_path_s)
        .arg(link_name_s)
        .output();
    assert!(c.is_ok());

    let a = format!(" ─┬ {}", dir_s);
    let b = format!("  ├── {}", file_path_s);
    let c = format!("  └── {}", link_name_s);

    assert_cli::Assert::main_binary()
        .with_args(&["-p", &dir_s])
        .stdout()
        .contains(a.as_str())
        .stdout()
        .contains(b.as_str())
        .stdout()
        .contains(c.as_str())
        .unwrap();
}

// Hard links are ignored as the inode is the same as the file
// fix! [rivy; 2020-01-22] may fail on "usual" windows hosts as `ln` is not usually an available command
#[test]
pub fn test_hard_sym_link() {
    let dir = Builder::new().tempdir().unwrap();
    let file = build_temp_file(&dir);
    let dir_s = dir.path().to_str().unwrap();
    let file_path_s = file.to_str().unwrap();

    let link_name = dir.path().join("the_link");
    let link_name_s = link_name.to_str().unwrap();
    let c = Command::new("ln")
        .arg(file_path_s)
        .arg(link_name_s)
        .output();
    assert!(c.is_ok());

    let a = format!(" ─┬ {}", dir_s);
    let b = format!("  └── {}", link_name_s);
    let b2 = format!("  └── {}", file_path_s);

    // Because this is a hard link the file and hard link look identical. Therefore
    // we cannot guarantee which version will appear first.
    let result = panic::catch_unwind(|| {
        assert_cli::Assert::main_binary()
            .with_args(&["-p", dir_s])
            .stdout()
            .contains(a.as_str())
            .stdout()
            .contains(b.as_str())
            .unwrap();
    });
    if result.is_err() {
        assert_cli::Assert::main_binary()
            .with_args(&["-p", dir_s])
            .stdout()
            .contains(a.as_str())
            .stdout()
            .contains(b2.as_str())
            .unwrap();
    }
}

// Check we don't recurse down an infinite symlink tree
// fix! [rivy; 2020-01-22] possible on "windows"?; `ln` is not usually an available command; creation of symbolic links requires special enhanced permissions
//  ... ref: <https://superuser.com/questions/343074/directory-junction-vs-directory-symbolic-link> @@ <https://archive.is/gpTLE>
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_recursive_sym_link() {
    let dir = Builder::new().tempdir().unwrap();
    let dir_s = dir.path().to_str().unwrap();

    let link_name = dir.path().join("the_link");
    let link_name_s = link_name.to_str().unwrap();

    let c = Command::new("ln")
        .arg("-s")
        .arg(dir_s)
        .arg(link_name_s)
        .output();
    assert!(c.is_ok());

    let a = format!(" ─┬ {}", dir_s);
    let b = format!("  └── {}", link_name_s);

    assert_cli::Assert::main_binary()
        .with_args(&["-p", dir_s])
        .stdout()
        .contains(a.as_str())
        .stdout()
        .contains(b.as_str())
        .unwrap();
}

// Check against directories and files whos names are substrings of each other
// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_substring_of_names() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "src/test_dir2"])
        .stdout()
        .is(no_substring_of_names_output().as_str())
        .unwrap();
}

#[cfg(target_os = "linux")]
fn no_substring_of_names_output() -> String {
    "
  24K ─┬ test_dir2
 8.0K  ├─┬ dir
 4.0K  │ └── hello
 8.0K  ├─┬ dir_substring
 4.0K  │ └── hello
 4.0K  └── dir_name_clash
    "
    .into()
}

#[cfg(target_os = "macos")]
fn no_substring_of_names_output() -> String {
    "
  12K ─┬ test_dir2
 4.0K  ├─┬ dir
 4.0K  │ └── hello
 4.0K  ├── dir_name_clash
 4.0K  └─┬ dir_substring
 4.0K    └── hello
    "
    .into()
}

#[cfg(target_os = "windows")]
fn no_substring_of_names_output() -> String {
    "
  16B ─┬ test_dir2
   6B  ├─┬ dir_substring
   6B  │ └── hello
   5B  ├─┬ dir
   5B  │ └── hello
   5B  └── dir_name_clash
    "
    .into()
}

// Check against directories and files whos names are substrings of each other
#[test]
pub fn test_ignore_dir() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "-X", "dir_substring", "src/test_dir2"])
        .stdout()
        .is(ignore_dir_output().as_str())
        .unwrap();
}

#[cfg(target_os = "linux")]
fn ignore_dir_output() -> String {
    "
  16K ─┬ test_dir2
 8.0K  ├─┬ dir
 4.0K  │ └── hello
 4.0K  └── dir_name_clash
    "
    .into()
}

#[cfg(target_os = "macos")]
fn ignore_dir_output() -> String {
    "
 8.0K ─┬ test_dir2
 4.0K  ├─┬ dir
 4.0K  │ └── hello
 4.0K  └── dir_name_clash
    "
    .into()
}

#[cfg(target_os = "windows")]
fn ignore_dir_output() -> String {
    "
  10B ─┬ test_dir2
   5B  ├─┬ dir
   5B  │ └── hello
   5B  └── dir_name_clash
    "
    .into()
}
