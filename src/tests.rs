extern crate ansi_term;
extern crate tempfile;
use self::tempfile::Builder;
use self::tempfile::TempDir;
use super::*;
use display::format_string;
use std::fs::File;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::process::Command;

#[test]
pub fn test_main() {
    assert_cli::Assert::main_binary()
        .with_args(&["src/test_dir"])
        .stdout()
        .is(main_output(true))
        .unwrap();
}

#[test]
pub fn test_main_long_paths() {
    assert_cli::Assert::main_binary()
        .with_args(&["-p", "src/test_dir"])
        .stdout()
        .is(main_output(false))
        .unwrap();
}

#[cfg(target_os = "macos")]
fn main_output(short_paths: bool) -> String {
    format!(
        "{}
{}
{}
{}",
        format_string("src/test_dir", true, short_paths, " 4.0K", ""),
        format_string("src/test_dir/many", true, short_paths, " 4.0K", "└─┬",),
        format_string("src/test_dir/many/hello_file", true, short_paths, " 4.0K", "  ├──",),
        format_string("src/test_dir/many/a_file", false, short_paths, "   0B", "  └──",),
    )
}

#[cfg(target_os = "linux")]
fn main_output(short_paths: bool) -> String {
    format!(
        "{}
{}
{}
{}",
        format_string("src/test_dir", true, short_paths, "  12K", ""),
        format_string("src/test_dir/many", true, short_paths, " 8.0K", "└─┬",),
        format_string("src/test_dir/many/hello_file", true, short_paths, " 4.0K", "  ├──",),
        format_string("src/test_dir/many/a_file", false, short_paths, "   0B", "  └──",),
    )
}

#[test]
pub fn test_apparent_size() {
    let r = format!(
        "{}",
        format_string("src/test_dir/many/hello_file", true, true, "   6B", "  ├──",),
    );

    assert_cli::Assert::main_binary()
        .with_args(&["-s", "src/test_dir"])
        .stdout()
        .contains(r)
        .unwrap();
}

fn build_temp_file(dir: &TempDir) -> (PathBuf) {
    let file_path = dir.path().join("notes.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "I am a temp file").unwrap();
    file_path
}

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

    assert_cli::Assert::main_binary()
        .with_args(&[dir_s])
        .stdout()
        .contains(soft_sym_link_output(dir_s, file_path_s, link_name_s))
        .unwrap();
}

#[cfg(target_os = "macos")]
fn soft_sym_link_output(dir: &str, file_path: &str, link_name: &str) -> String {
    format!(
        "{}
{}
{}",
        format_string(dir, true, true, " 8.0K", ""),
        format_string(file_path, true, true, " 4.0K", "├──",),
        format_string(link_name, false, true, " 4.0K", "└──",),
    )
}
#[cfg(target_os = "linux")]
fn soft_sym_link_output(dir: &str, file_path: &str, link_name: &str) -> String {
    format!(
        "{}
{}
{}",
        format_string(dir, true, true, " 8.0K", ""),
        format_string(file_path, true, true, " 4.0K", "├──",),
        format_string(link_name, false, true, "   0B", "└──",),
    )
}

// Hard links are ignored as the inode is the same as the file
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

    let (r, r2) = hard_link_output(dir_s, file_path_s, link_name_s);

    // Because this is a hard link the file and hard link look identicle. Therefore
    // we cannot guarantee which version will appear first.
    // TODO: Consider adding predictable itteration order (sort file entries by name?)
    let result = panic::catch_unwind(|| {
        assert_cli::Assert::main_binary()
            .with_args(&[dir_s])
            .stdout()
            .contains(r)
            .unwrap();
    });
    if result.is_err() {
        assert_cli::Assert::main_binary()
            .with_args(&[dir_s])
            .stdout()
            .contains(r2)
            .unwrap();
    }
}

#[cfg(target_os = "macos")]
fn hard_link_output(dir_s: &str, file_path_s: &str, link_name_s: &str) -> (String, String) {
    let r = format!(
        "{}
{}",
        format_string(dir_s, true, true, " 4.0K", ""),
        format_string(file_path_s, true, true, " 4.0K", "└──")
    );
    let r2 = format!(
        "{}
{}",
        format_string(dir_s, true, true, " 4.0K", ""),
        format_string(link_name_s, true, true, " 4.0K", "└──")
    );
    (r, r2)
}

#[cfg(target_os = "linux")]
fn hard_link_output(dir_s: &str, file_path_s: &str, link_name_s: &str) -> (String, String) {
    let r = format!(
        "{}
{}",
        format_string(dir_s, true, true, " 8.0K", ""),
        format_string(file_path_s, true, true, " 4.0K", "└──")
    );
    let r2 = format!(
        "{}
{}",
        format_string(dir_s, true, true, " 8.0K", ""),
        format_string(link_name_s, true, true, " 4.0K", "└──")
    );
    (r, r2)
}

//Check we don't recurse down an infinite symlink tree
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

    assert_cli::Assert::main_binary()
        .with_args(&[dir_s])
        .stdout()
        .contains(recursive_sym_link_output(dir_s, link_name_s))
        .unwrap();
}

#[cfg(target_os = "macos")]
fn recursive_sym_link_output(dir: &str, link_name: &str) -> String {
    format!(
        "{}
{}",
        format_string(dir, true, true, " 4.0K", ""),
        format_string(link_name, true, true, " 4.0K", "└──",),
    )
}
#[cfg(target_os = "linux")]
fn recursive_sym_link_output(dir: &str, link_name: &str) -> String {
    format!(
        "{}
{}",
        format_string(dir, true, true, " 4.0K", ""),
        format_string(link_name, true, true, "   0B", "└──",),
    )
}

// TODO: add test for bad path
