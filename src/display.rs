extern crate ansi_term;

use self::ansi_term::Colour::Fixed;
use self::ansi_term::Style;
use utils::Node;

static UNITS: [char; 4] = ['T', 'G', 'M', 'K'];

pub struct DisplayData {
    pub short_paths: bool,
    pub is_reversed: bool,
}

impl DisplayData {
    fn get_first_chars(&self) -> &str {
        if self.is_reversed {
            "─┴"
        } else {
            "─┬"
        }
    }

    fn get_tree_chars(
        &self,
        num_siblings: u64,
        max_siblings: u64,
        has_children: bool,
    ) -> &'static str {
        if self.is_reversed {
            if num_siblings == max_siblings - 1 {
                if has_children {
                    "┌─┴"
                } else {
                    "┌──"
                }
            } else if has_children {
                "├─┴"
            } else {
                "├──"
            }
        } else {
            if num_siblings == 0 {
                if has_children {
                    "└─┬"
                } else {
                    "└──"
                }
            } else if has_children {
                "├─┬"
            } else {
                "├──"
            }
        }
    }

    fn biggest(&self, num_siblings: u64, max_siblings: u64) -> bool {
        if self.is_reversed {
            num_siblings == 0
        } else {
            num_siblings == max_siblings - 1
        }
    }
}

pub fn draw_it(
    permissions: bool,
    depth: Option<u64>,
    use_full_path: bool,
    is_reversed: bool,
    root_node: Node,
) {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }
    let display_data = DisplayData {
        short_paths: !use_full_path,
        is_reversed,
    };

    for c in root_node.children {
        let first_tree_chars = display_data.get_first_chars();
        display_node(*c, true, depth, first_tree_chars, &display_data)
    }
}

fn display_node(
    node: Node,
    is_biggest: bool,
    depth: Option<u64>,
    indent: &str,
    display_data: &DisplayData,
) {
    let new_depth = match depth {
        None => None,
        Some(0) => return,
        Some(d) => Some(d - 1),
    };
    let short = display_data.short_paths;
    print_this_node(&node, is_biggest, short, indent);

    let mut num_siblings = node.children.len() as u64;
    let max_sibling = num_siblings;
    let new_indent = clean_indentation_string(indent);

    for c in node.children {
        num_siblings -= 1;
        let chars = display_data.get_tree_chars(num_siblings, max_sibling, c.children.len() > 0);
        let is_biggest = display_data.biggest(num_siblings, max_sibling);
        let full_indent = (new_indent.clone() + chars);
        display_node(*c, is_biggest, new_depth, &*full_indent, display_data)
    }
}

fn clean_indentation_string(s: &str) -> String {
    let mut is: String = s.into();
    // For reversed:
    is = is.replace("┌─┴", "  ");
    is = is.replace("┌──", "  ");
    is = is.replace("├─┴", "│ ");
    is = is.replace("─┴", " ");
    // For normal
    is = is.replace("└─┬", "  ");
    is = is.replace("└──", "  ");
    is = is.replace("├─┬", "│ ");
    is = is.replace("─┬", " ");
    // For both
    is = is.replace("├──", "│ ");
    is
}

fn print_this_node(node: &Node, is_biggest: bool, short_paths: bool, indentation: &str) {
    let pretty_size = format!("{:>5}", human_readable_number(node.size),);
    println!(
        "{}",
        format_string(
            &*node.name,
            is_biggest,
            short_paths,
            pretty_size.as_ref(),
            indentation
        )
    )
}
// idea: squash these 2
pub fn format_string(
    dir_name: &str,
    is_biggest: bool,
    short_paths: bool,
    size: &str,
    indentation: &str,
) -> String {
    let printable_name = {
        if short_paths {
            dir_name.split('/').last().unwrap_or(dir_name)
        } else {
            dir_name
        }
    };
    format!(
        "{} {} {}",
        if is_biggest {
            Fixed(196).paint(size)
        } else {
            Style::new().paint(size)
        },
        indentation,
        printable_name,
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
