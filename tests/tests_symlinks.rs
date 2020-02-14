use std::fs::File;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;
use tempfile::TempDir;

// File sizes differ on both platform and on the format of the disk.

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

    let c = format!(" ┌── {}", link_name_s);
    let b = format!(" ├── {}", file_path_s);
    let a = format!("─┴ {}", dir_s);

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
    let b = format!("  └── {}", link_name_s);

    assert_cli::Assert::main_binary()
        .with_args(&["-r", "-p", dir_s])
        .stdout()
        .contains(a.as_str())
        .stdout()
        .contains(b.as_str())
        .unwrap();
}
