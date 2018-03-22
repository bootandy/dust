extern crate ansi_term;

use dust::Node;
use std::cmp;
use self::ansi_term::Colour::Fixed;

pub fn display(permissions: bool, to_display: &Vec<&Node>) -> () {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }

    display_node(to_display[0], &to_display, true, 1, "")
}

fn display_node<S: Into<String>>(
    node_to_print: &Node,
    to_display: &Vec<&Node>,
    is_first: bool,
    depth: u8,
    indentation_str: S,
) {
    let mut is = indentation_str.into();
    print_this_node(node_to_print, is_first, depth, is.as_ref());

    is = is.replace("└─┬", "  ");
    is = is.replace("└──", "  ");
    is = is.replace("├──", "│ ");
    is = is.replace("├─┬", "│ ");

    let printable_node_slashes = node_to_print.entry().name().matches('/').count();

    let mut num_siblings = to_display.iter().fold(0, |a, b| {
        if node_to_print.children().contains(b)
            && b.entry().name().matches('/').count() == printable_node_slashes + 1
        {
            a + 1
        } else {
            a
        }
    });

    let mut is_biggest = true;
    for node in to_display {
        if node_to_print.children().contains(node) {
            let has_display_children = node.children()
                .iter()
                .fold(false, |has_kids, n| has_kids || to_display.contains(&n));

            let has_children = node.children().len() > 0 && has_display_children;
            if node.entry().name().matches('/').count() == printable_node_slashes + 1 {
                num_siblings -= 1;

                let tree_chars = {
                    if num_siblings == 0 {
                        if has_children {
                            "└─┬"
                        } else {
                            "└──"
                        }
                    } else {
                        if has_children {
                            "├─┬"
                        } else {
                            "├──"
                        }
                    }
                };
                display_node(
                    &node,
                    to_display,
                    is_biggest,
                    depth + 1,
                    is.to_string() + tree_chars,
                );
                is_biggest = false;
            }
        }
    }
}

fn print_this_node(node_to_print: &Node, is_biggest: bool, depth: u8, indentation_str: &str) {
    let padded_size = format!("{:>5}", human_readable_number(node_to_print.entry().size()),);
    println!(
        "{} {} {}",
        if is_biggest {
            Fixed(196).paint(padded_size)
        } else {
            Fixed(7).paint(padded_size)
        },
        indentation_str,
        Fixed(7)
            .on(Fixed(cmp::min(8, (depth) as u8) + 231))
            .paint(node_to_print.entry().name().to_string())
    );
}

fn human_readable_number(size: u64) -> (String) {
    let units = vec!["T", "G", "M", "K"]; //make static

    for (i, u) in units.iter().enumerate() {
        let marker = 1024u64.pow((units.len() - i) as u32);
        if size >= marker {
            if size / marker < 10 {
                return format!("{:.1}{}", (size as f32 / marker as f32), u);
            } else {
                return format!("{}{}", (size / marker), u);
            }
        }
    }
    return format!("{}B", size);
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_human_readable_number() {
        assert_eq!(human_readable_number(1), "1B");
        assert_eq!(human_readable_number(956), "956B");
        assert_eq!(human_readable_number(1004), "1004B");
        assert_eq!(human_readable_number(1024), "1.0K");
        assert_eq!(human_readable_number(1536), "1.5K");
        assert_eq!(human_readable_number(1024 * 512), "512K");
        assert_eq!(human_readable_number(1024 * 1024), "1.0M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 - 1), "1023M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 20), "20G");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 1024), "1.0T");
    }
}
