extern crate ansi_term;
extern crate tempfile;
use self::tempfile::Builder;
use self::tempfile::TempDir;
use super::*;
use display::format_string;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[test]
pub fn test_main() {
    let r = format!(
        "{}
{}
{}
{}",
        format_string("src/test_dir", true, " 4.0K", ""),
        format_string("src/test_dir/many", true, " 4.0K", "└─┬",),
        format_string("src/test_dir/many/hello_file", true, " 4.0K", "  ├──",),
        format_string("src/test_dir/many/a_file", false, "   0B", "  └──",),
    );

    assert_cli::Assert::main_binary()
        .with_args(&["src/test_dir"])
        .stdout()
        .is(r)
        .unwrap();
}

#[test]
pub fn test_apparent_size() {
    let r = format!(
        "{}",
        format_string("src/test_dir/many/hello_file", true, "   6B", "  ├──",),
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

    let r = format!(
        "{}
{}
{}",
        format_string(dir_s, true, " 8.0K", ""),
        format_string(file_path_s, true, " 4.0K", "├──",),
        format_string(link_name_s, false, " 4.0K", "└──",),
    );

    assert_cli::Assert::main_binary()
        .with_args(&[dir_s])
        .stdout()
        .contains(r)
        .unwrap();
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

    let r = format!(
        "{}
{}",
        format_string(dir_s, true, " 4.0K", ""),
        format_string(file_path_s, true, " 4.0K", "└──")
    );

    assert_cli::Assert::main_binary()
        .with_args(&[dir_s])
        .stdout()
        .contains(r)
        .unwrap();
}

//Check we don't recurse down an infinite symlink tree
#[test]
pub fn test_recursive_sym_link() {
    let dir = Builder::new().tempdir().unwrap();
    let dir_s = dir.path().to_str().unwrap();

    let link_name = dir.path().join("the_link");
    let link_name_s = link_name.to_str().unwrap();

    let c = Command::new("cd").arg(dir_s).output();
    assert!(c.is_ok());
    let c = Command::new("ln")
        .arg("-s")
        .arg(".")
        .arg(link_name_s)
        .output();
    assert!(c.is_ok());

    let r = format!("{}", format_string(dir_s, true, " 4.0K", ""));

    assert_cli::Assert::main_binary()
        .with_args(&[dir_s])
        .stdout()
        .contains(r)
        .unwrap();
}
