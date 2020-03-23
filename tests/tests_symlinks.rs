use assert_cmd::Command;
use std::fs::File;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::str;
use tempfile::Builder;
use tempfile::TempDir;

// File sizes differ on both platform and on the format of the disk.
// Windows: `ln` is not usually an available command; creation of symbolic links requires special enhanced permissions

fn build_temp_file(dir: &TempDir) -> PathBuf {
    let file_path = dir.path().join("notes.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "I am a temp file").unwrap();
    file_path
}

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

    let c = format!(" ┌── {}", link_name_s);
    let b = format!(" ├── {}", file_path_s);
    let a = format!("─┴ {}", dir_s);

    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd.arg("-p").arg("-c").arg(dir_s).unwrap().stdout;

    let output = str::from_utf8(&output).unwrap();

    assert!(output.contains(a.as_str()));
    assert!(output.contains(b.as_str()));
    assert!(output.contains(c.as_str()));
}

#[cfg_attr(target_os = "windows", ignore)]
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

    let link_output = format!(" ┌── {}", link_name_s);
    let file_output = format!(" ┌── {}", file_path_s);
    let dirs_output = format!("─┴ {}", dir_s);

    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd.arg("-p").arg("-c").arg(dir_s).unwrap().stdout;

    // Because this is a hard link the file and hard link look identical. Therefore
    // we cannot guarantee which version will appear first.
    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains(dirs_output.as_str()));
    assert!(output.contains(link_output.as_str()) || output.contains(file_output.as_str()));
}

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

    let a = format!("─┬ {}", dir_s);
    let b = format!(" └── {}", link_name_s);

    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd.arg("-p").arg("-c").arg("-r").arg(dir_s).unwrap().stdout;

    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains(a.as_str()));
    assert!(output.contains(b.as_str()));
}
