extern crate ansi_term;
use super::*;
use display::format_string;

// TESTS TODO:
// handle recursive dirs
// handle soft links
// handle hard links

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
