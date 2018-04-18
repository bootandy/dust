extern crate ansi_term;

use self::ansi_term::Colour::Fixed;

use lib::Node;

static UNITS: [char; 4] = ['T', 'G', 'M', 'K'];

pub fn draw_it(permissions: bool, heads: &Vec<Node>, to_display: &Vec<&Node>) -> () {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }

    for d in to_display {
        if heads.contains(d) {
            display_node(d, &to_display, true, "")
        }
    }
}

fn display_node<S: Into<String>>(
    node_to_print: &Node,
    to_display: &Vec<&Node>,
    is_first: bool,
    indentation_str: S,
) {
    let mut is = indentation_str.into();
    print_this_node(node_to_print, is_first, is.as_ref());

    is = is.replace("└─┬", "  ");
    is = is.replace("└──", "  ");
    is = is.replace("├──", "│ ");
    is = is.replace("├─┬", "│ ");

    let printable_node_slashes = node_to_print.name().matches('/').count();

    let mut num_siblings = to_display.iter().fold(0, |a, b| {
        if node_to_print.children().contains(b)
            && b.name().matches('/').count() == printable_node_slashes + 1
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
            if node.name().matches('/').count() == printable_node_slashes + 1 {
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
                display_node(&node, to_display, is_biggest, is.to_string() + tree_chars);
                is_biggest = false;
            }
        }
    }
}

fn print_this_node(node: &Node, is_biggest: bool, indentation: &str) {
    let pretty_size = format!("{:>5}", human_readable_number(node.size()),);
    println!(
        "{}",
        format_string(node.name(), is_biggest, pretty_size.as_ref(), indentation)
    )
}

pub fn format_string(dir_name: &str, is_biggest: bool, size: &str, indentation: &str) -> String {
    format!(
        "{} {} {}",
        if is_biggest {
            Fixed(196).paint(size)
        } else {
            Fixed(7).paint(size)
        },
        indentation,
        dir_name,
    )
}

fn human_readable_number(size: u64) -> String {
    for (i, u) in UNITS.iter().enumerate() {
        let marker = 1024u64.pow((UNITS.len() - i) as u32);
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
