mod tests_symlinks;

// File sizes differ on both platform and on the format of the disk.
// We can at least test the file names are there
#[test]
pub fn test_basic_output() {
    assert_cli::Assert::main_binary()
        .with_args(&["src/test_dir/"])
        .stdout()
        .contains(" ┌─┴ test_dir ")
        .stdout()
        .contains("  ┌─┴ many ")
        .stdout()
        .contains("    ├── hello_file ")
        .stdout()
        .contains("     ┌── a_file ")
        .unwrap();
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_basic() {
    // -c is no color mode - This makes testing much simpler
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "src/test_dir"])
        .stdout()
        .is(main_output().as_str())
        .unwrap();
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_multi_arg() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "src/test_dir/many/", "src/test_dir/", "src/test_dir"])
        .stdout()
        .is(main_output().as_str())
        .unwrap();
}

#[cfg(target_os = "macos")]
fn main_output() -> String {
    r#"
   0B     ┌── a_file       │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── hello_file   │████████████████████████████████████████████ │ 100%
 4.0K   ┌─┴ many           │████████████████████████████████████████████ │ 100%
 4.0K ┌─┴ test_dir         │████████████████████████████████████████████ │ 100%"#
        .to_string()
}

#[cfg(target_os = "linux")]
fn main_output() -> String {
    r#"
   0B     ┌── a_file       │              ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── hello_file   │              ░░░░░░░░░░░░░░░███████████████ │  33%
 8.0K   ┌─┴ many           │              ██████████████████████████████ │  67%
  12K ┌─┴ test_dir         │████████████████████████████████████████████ │ 100%
    "#
    .to_string()
}

#[cfg(target_os = "windows")]
fn main_output() -> String {
    "PRs welcome".to_string()
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_main_long_paths() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "-p", "src/test_dir"])
        .stdout()
        .is(main_output_long_paths().as_str())
        .unwrap();
}

#[cfg(target_os = "macos")]
fn main_output_long_paths() -> String {
    r#"
   0B     ┌── src/test_dir/many/a_file       │░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── src/test_dir/many/hello_file   │██████████████████████████ │ 100%
 4.0K   ┌─┴ src/test_dir/many                │██████████████████████████ │ 100%
 4.0K ┌─┴ src/test_dir                       │██████████████████████████ │ 100%  
 "#
    .to_string()
}

#[cfg(target_os = "linux")]
fn main_output_long_paths() -> String {
    r#"   
   0B     ┌── src/test_dir/many/a_file       │        ░░░░░░░░░░░░░░░░░█ │   0%
 4.0K     ├── src/test_dir/many/hello_file   │        ░░░░░░░░░█████████ │  33%
 8.0K   ┌─┴ src/test_dir/many                │        ██████████████████ │  67%
  12K ┌─┴ src/test_dir                       │██████████████████████████ │ 100%    
    "#
    .to_string()
}

#[cfg(target_os = "windows")]
fn main_output_long_paths() -> String {
    "PRs welcome".to_string()
}

// fix! [rivy; 2020-22-01] "windows" result data can vary by host (size seems to be variable by one byte); fix code vs test and re-enable
#[cfg_attr(target_os = "windows", ignore)]
#[test]
pub fn test_apparent_size() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "-s", "src/test_dir"])
        .stdout()
        .is(output_apparent_size().as_str())
        .unwrap();
}

#[cfg(target_os = "linux")]
fn output_apparent_size() -> String {
    r#"
   0B     ┌── a_file       │                     ░░░░░░░░░░░░░░░░░░░░░░█ │   0%
   6B     ├── hello_file   │                     ░░░░░░░░░░░░░░░░░░░░░░█ │   0%
 4.0K   ┌─┴ many           │                     ███████████████████████ │  50%
 8.0K ┌─┴ test_dir         │████████████████████████████████████████████ │ 100%
    "#
    .to_string()
}

#[cfg(target_os = "macos")]
fn output_apparent_size() -> String {
    r#"
   0B     ┌── a_file       │                  ░░░░░░░░░░░░░░░░░░░░░░░░░█ │   0%
   6B     ├── hello_file   │                  ░░░░░░░░░░░░░░░░░░░░░░░░██ │   3%
 134B   ┌─┴ many           │                  ██████████████████████████ │  58%
 230B ┌─┴ test_dir         │████████████████████████████████████████████ │ 100%
    "#
    .to_string()
}

#[cfg(target_os = "windows")]
fn output_apparent_size() -> String {
    "".to_string()
}

#[test]
pub fn test_reverse_flag() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "-r", "src/test_dir/"])
        .stdout()
        .contains(" └─┬ test_dir ")
        .stdout()
        .contains("  └─┬ many ")
        .stdout()
        .contains("    ├── hello_file ")
        .stdout()
        .contains("    └── a_file ")
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
 4.0K   ┌── dir_name_clash  │                                   ████████ │  17%
 4.0K   │ ┌── hello         │                            ░░░░░░░████████ │  17%
 8.0K   ├─┴ dir_substring   │                            ███████████████ │  33%
 4.0K   │ ┌── hello         │                            ░░░░░░░████████ │  17%
 8.0K   ├─┴ dir             │                            ███████████████ │  33%
  24K ┌─┴ test_dir2         │███████████████████████████████████████████ │ 100%
    "
    .into()
}

#[cfg(target_os = "macos")]
fn no_substring_of_names_output() -> String {
    "
 4.0K     ┌── hello         │                            ███████████████ │  33%
 4.0K   ┌─┴ dir_substring   │                            ███████████████ │  33%
 4.0K   ├── dir_name_clash  │                            ███████████████ │  33%
 4.0K   │ ┌── hello         │                            ███████████████ │  33%
 4.0K   ├─┴ dir             │                            ███████████████ │  33%
  12K ┌─┴ test_dir2         │███████████████████████████████████████████ │ 100%
  "
    .into()
}

#[cfg(target_os = "windows")]
fn no_substring_of_names_output() -> String {
    "PRs".into()
}

// Check against directories and files whos names are substrings of each other
#[test]
pub fn test_ignore_dir() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "-X", "dir_substring", "src/test_dir2"])
        .stdout()
        .doesnt_contain("dir_substring")
        .unwrap();
}
