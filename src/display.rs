extern crate ansi_term;

use self::ansi_term::Colour::Fixed;
use self::ansi_term::Style;
use std::cmp::max;
use std::collections::HashSet;
use utils::{ensure_end_slash, strip_end_slash_including_root};

static UNITS: [char; 4] = ['T', 'G', 'M', 'K'];

pub struct DisplayData {
    pub short_paths: bool,
    pub is_reversed: bool,
    pub to_display: Vec<(String, u64)>,
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

    fn get_size(&self, node_to_print: &str) -> Option<u64> {
        for &(ref k, ref v) in self.to_display.iter() {
            if *k == *node_to_print {
                return Some(*v);
            }
        }
        None
    }

    fn count_siblings(&self, num_slashes: usize, ntp: &str) -> u64 {
        self.to_display.iter().fold(0, |a, b| {
            if b.0.starts_with(ntp) && b.0.as_str().matches('/').count() == num_slashes + 1 {
                a + 1
            } else {
                a
            }
        })
    }

    fn has_children(&self, new_depth: Option<u64>, ntp: &str, num_slashes: usize) -> bool {
        // this shouldn't be needed we should have already stripped
        if new_depth.is_none() || new_depth.unwrap() != 1 {
            for &(ref k2, _) in self.to_display.iter() {
                let ntp_with_slash = String::from(ntp.to_owned() + "/");
                if k2.starts_with(ntp_with_slash.as_str())
                    && k2.matches('/').count() == num_slashes + 1
                {
                    return true;
                }
            }
        }
        false
    }
}

pub fn draw_it(
    permissions: bool,
    depth: Option<u64>,
    base_dirs: HashSet<String>,
    display_data: &DisplayData,
) {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }
    let first_tree_chars = display_data.get_first_chars();
    let mut found = HashSet::new();

    for &(ref k, _) in display_data.to_display.iter() {
        if base_dirs.contains(k) {
            display_node(&k, &mut found, true, depth, first_tree_chars, display_data);
        }
    }
}

fn display_node(
    node: &str,
    nodes_already_found: &mut HashSet<String>,
    is_biggest: bool,
    depth: Option<u64>,
    indent: &str,
    display_data: &DisplayData,
) {
    if nodes_already_found.contains(node) {
        return;
    }
    nodes_already_found.insert(node.to_string());

    let new_depth = match depth {
        None => None,
        Some(0) => return,
        Some(d) => Some(d - 1),
    };

    match display_data.get_size(node) {
        None => println!("Can not find path: {}", node),
        Some(size) => {
            let short_path = display_data.short_paths;
            // move this inside display_data?
            if !display_data.is_reversed {
                print_this_node(node, size, is_biggest, short_path, indent);
            }
            fan_out(node, nodes_already_found, new_depth, indent, display_data);
            if display_data.is_reversed {
                print_this_node(node, size, is_biggest, short_path, indent);
            }
        }
    }
}

fn fan_out(
    node_to_print: &str,
    nodes_already_found: &mut HashSet<String>,
    new_depth: Option<u64>,
    indentation_str: &str,
    display_data: &DisplayData,
) {
    let new_indent = clean_indentation_string(indentation_str);
    let num_slashes = strip_end_slash_including_root(node_to_print)
        .matches('/')
        .count();

    let mut num_siblings = display_data.count_siblings(num_slashes, node_to_print);
    let max_siblings = num_siblings;

    for &(ref k, _) in display_data.to_display.iter() {
        let temp = String::from(ensure_end_slash(node_to_print));
        if k.starts_with(temp.as_str()) && k.matches('/').count() == num_slashes + 1 {
            num_siblings -= 1;
            let has_children = display_data.has_children(new_depth, k, num_slashes + 1);
            let new_tree_chars =
                display_data.get_tree_chars(num_siblings, max_siblings, has_children);
            let biggest = display_data.biggest(num_siblings, max_siblings);
            display_node(
                k,
                nodes_already_found,
                biggest,
                new_depth,
                &*(new_indent.to_string() + new_tree_chars),
                display_data,
            );
        }
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

fn print_this_node(
    node_name: &str,
    size: u64,
    is_biggest: bool,
    short_paths: bool,
    indentation: &str,
) {
    let pretty_size = format!("{:>5}", human_readable_number(size),);
    println!(
        "{}",
        format_string(
            node_name,
            is_biggest,
            short_paths,
            pretty_size.as_ref(),
            indentation
        )
    )
}

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
