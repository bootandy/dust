use std::fs::File;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::process::Command;
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

    assert_cli::Assert::main_binary()
        .with_args(&["-s", "-p", "-c", &dir_s])
        .stdout()
        .contains(a.as_str())
        .stdout()
        .contains(b.as_str())
        .stdout()
        .contains(c.as_str())
        .unwrap();
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

    let a = format!("─┴ {}", dir_s);
    let b = format!(" ┌── {}", link_name_s);
    let b2 = format!(" ┌── {}", file_path_s);

    // Because this is a hard link the file and hard link look identical. Therefore
    // we cannot guarantee which version will appear first.
    let result = panic::catch_unwind(|| {
        assert_cli::Assert::main_binary()
            .with_args(&["-p", "-c", dir_s])
            .stdout()
            .contains(a.as_str())
            .stdout()
            .contains(b.as_str())
            .unwrap();
    });
    if result.is_err() {
        assert_cli::Assert::main_binary()
            .with_args(&["-p", "-c", dir_s])
            .stdout()
            .contains(a.as_str())
            .stdout()
            .contains(b2.as_str())
            .unwrap();
    }
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

    let b = format!(" └── {}", dir_s);

    assert_cli::Assert::main_binary()
        .with_args(&["-s", "-c", "-r", "-p", dir_s])
        .stdout()
        .contains(b.as_str())
        .unwrap();
}
