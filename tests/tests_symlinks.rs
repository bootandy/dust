use assert_cmd::Command;
use std::fs::File;
use std::io::Write;
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

fn link_it(link_path: PathBuf, file_path_s: &str, is_soft: bool) -> String {
    let link_name_s = link_path.to_str().unwrap();
    let mut c = Command::new("ln");
    if is_soft {
        c.arg("-s");
    }
    c.arg(file_path_s);
    c.arg(link_name_s);
    assert!(c.output().is_ok());
    return link_name_s.into();
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_soft_sym_link() {
    let dir = Builder::new().tempdir().unwrap();
    let file = build_temp_file(&dir);
    let dir_s = dir.path().to_str().unwrap();
    let file_path_s = file.to_str().unwrap();

    let link_name = dir.path().join("the_link");
    let link_name_s = link_it(link_name, file_path_s, true);

    let c = format!(" ├── {}", link_name_s);
    let b = format!(" ┌── {}", file_path_s);
    let a = format!("─┴ {}", dir_s);

    let mut cmd = Command::cargo_bin("dust").unwrap();
    // Mac test runners create long filenames in tmp directories
    let output = cmd
        .args(["-p", "-c", "-s", "-w 999", dir_s])
        .unwrap()
        .stdout;

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
    link_it(link_name, file_path_s, false);

    let file_output = format!(" ┌── {}", file_path_s);
    let dirs_output = format!("─┴ {}", dir_s);

    let mut cmd = Command::cargo_bin("dust").unwrap();
    // Mac test runners create long filenames in tmp directories
    let output = cmd.args(["-p", "-c", "-w 999", dir_s]).unwrap().stdout;

    // The link should not appear in the output because multiple inodes are now ordered
    // then filtered.
    let output = str::from_utf8(&output).unwrap();
    assert!(output.contains(dirs_output.as_str()));
    assert!(output.contains(file_output.as_str()));
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_hard_sym_link_no_dup_multi_arg() {
    let dir = Builder::new().tempdir().unwrap();
    let dir_link = Builder::new().tempdir().unwrap();
    let file = build_temp_file(&dir);
    let dir_s = dir.path().to_str().unwrap();
    let dir_link_s = dir_link.path().to_str().unwrap();
    let file_path_s = file.to_str().unwrap();

    let link_name = dir_link.path().join("the_link");
    let link_name_s = link_it(link_name, file_path_s, false);

    let mut cmd = Command::cargo_bin("dust").unwrap();

    // Mac test runners create long filenames in tmp directories
    let output = cmd
        .args(["-p", "-c", "-w 999", "-b", dir_link_s, dir_s])
        .unwrap()
        .stdout;

    // The link or the file should appeart but not both
    let output = str::from_utf8(&output).unwrap();
    println!("cmd:\n{:?}", cmd);
    println!("output:\n{:?}", output);
    let has_file_only = output.contains(file_path_s) && !output.contains(&link_name_s);
    let has_link_only = !output.contains(file_path_s) && output.contains(&link_name_s);
    assert!(has_file_only || has_link_only)
}

#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_recursive_sym_link() {
    let dir = Builder::new().tempdir().unwrap();
    let dir_s = dir.path().to_str().unwrap();

    let link_name = dir.path().join("the_link");
    let link_name_s = link_it(link_name, dir_s, true);

    let a = format!("─┬ {}", dir_s);
    let b = format!(" └── {}", link_name_s);

    let mut cmd = Command::cargo_bin("dust").unwrap();
    let output = cmd
        .arg("-p")
        .arg("-c")
        .arg("-r")
        .arg("-s")
        .arg("-w 999")
        .arg(dir_s)
        .unwrap()
        .stdout;
    let output = str::from_utf8(&output).unwrap();

    assert!(output.contains(a.as_str()));
    assert!(output.contains(b.as_str()));
}
