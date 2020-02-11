extern crate ansi_term;

use self::ansi_term::Colour::Fixed;
use self::ansi_term::Style;
use crate::utils::Node;
use terminal_size::{terminal_size, Height, Width};

use std::cmp::max;
use std::iter::repeat;
use std::path::Path;

static UNITS: [char; 4] = ['T', 'G', 'M', 'K'];
static BLOCKS: [char; 5] = ['█', '▓', '▒', '░', ' '];

pub struct DisplayData {
    pub short_paths: bool,
    pub is_reversed: bool,
    pub colors_on: bool,
    pub terminal_size: Option<(Width, Height)>,
    pub base_size: u64,
    pub longest_string_length: usize,
}

impl DisplayData {

    #[allow(clippy::collapsible_if)]
    fn get_tree_chars(
        &self,
        was_i_last: bool,
        has_children: bool,
    ) -> &'static str {
        if self.is_reversed {
            if was_i_last {
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
            if was_i_last {
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

    fn is_biggest(&self, num_siblings: u64, max_siblings: u64) -> bool {
        if self.is_reversed {
            num_siblings == 0
        } else {
            num_siblings == max_siblings - 1
        }
    }

    fn is_last(&self, num_siblings: u64, max_siblings: u64) -> bool {
        if self.is_reversed {
            num_siblings == max_siblings - 1
        } else {
            num_siblings == 0
        }
    }

    fn get_children_from_node(&self, node: Node) -> impl Iterator<Item = Node> {
        if self.is_reversed {
            let n: Vec<Node> = node.children.into_iter().rev().map(|a| a).collect();
            n.into_iter()
        } else {
            node.children.into_iter()
        }
    }
}
    fn get_children_from_node_dup(node: Node, is_reversed: bool) -> impl Iterator<Item = Node> {
        if is_reversed {
            let n: Vec<Node> = node.children.into_iter().rev().map(|a| a).collect();
            n.into_iter()
        } else {
            node.children.into_iter()
        }
    }

pub fn draw_it(
    permissions: bool,
    use_full_path: bool,
    is_reversed: bool,
    no_colors: bool,
    root_node: Node,
) {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }

    let longest_string_length = find_longest_dir_name(&root_node, "", !use_full_path);

    let ww = {
        if let Some((Width(w), Height(_h))) = terminal_size() {
            w
        } else {
            80
        }
    } - 16;

    for c in get_children_from_node_dup(root_node, is_reversed) {
        let max_bar_length = ww as usize - longest_string_length;
        let bar_text = repeat(BLOCKS[0]).take(max_bar_length).collect::<String>();

        let display_data = DisplayData {
            short_paths: !use_full_path,
            is_reversed,
            colors_on: !no_colors,
            terminal_size: terminal_size(),
            base_size: c.size,
            longest_string_length,
        };

        display_node(
            c,
            true,
            true,
            "".to_string(),
            &display_data,
            bar_text,
        );
    }
}

fn find_longest_dir_name(node: &Node, indent: &str, long_paths: bool) -> usize {
    let mut longest = get_printable_name(node.name.clone(), long_paths, indent)
        .chars()
        .count();

    for c in node.children.iter() {
        // each tree drawing is 3 chars
        let full_indent: String = indent.to_string() + "   ";
        longest = max(
            longest,
            find_longest_dir_name(c, &*full_indent, long_paths),
        );
    }
    longest
}

// can we make 2 of these one for pre and post order traversal?
fn display_node(
    node: Node,
    is_biggest: bool,
    was_i_last: bool,
    new_indent: String,
    display_data: &DisplayData,
    parent_bar: String,
) {
    let name = node.name.clone();
    let size = node.size;

    let chars = display_data.get_tree_chars(was_i_last, !node.children.is_empty());
    let indent2 = new_indent.clone() + chars;

    let percent_size = size as f32 / display_data.base_size as f32;
    let bar_text = generate_bar(parent_bar, percent_size);

    if !display_data.is_reversed {
        print_this_node(
            &name,
            size,
            is_biggest,
            display_data,
            &*indent2,
            bar_text.as_ref(),
        );
    }

    let mut num_siblings = node.children.len() as u64;
    let max_sibling = num_siblings;
    for c in display_data.get_children_from_node(node) {
        num_siblings -= 1;
        // can we just do is first ? + handle reverse mode. consider equal values
        let is_biggest = display_data.is_biggest(num_siblings, max_sibling);
        let is_last = display_data.is_last(num_siblings, max_sibling);
        display_node(
            c,
            is_biggest,
            is_last,
            clean_indentation_string(&*indent2),
            display_data,
            bar_text.clone()
        );
    }

    if display_data.is_reversed {
        print_this_node(
            &name,
            size,
            is_biggest,
            display_data,
            &*indent2,
            bar_text.as_ref(),
        );
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

fn print_this_node<P: AsRef<Path>>(
    name: P,
    size: u64,
    is_biggest: bool,
    display_data: &DisplayData,
    indentation: &str,
    bar_text: &str,
) {
    println!(
        "{}",
        format_string(
            name,
            is_biggest,
            display_data,
            size,
            indentation,
            bar_text,
        )
    )
}

fn get_printable_name<P: AsRef<Path>>(
    dir_name: P,
    long_paths: bool,
    indentation: &str,
) -> String {
    let dir_name = dir_name.as_ref();
    let printable_name = {
        if long_paths {
            match dir_name.parent() {
                Some(prefix) => match dir_name.strip_prefix(prefix) {
                    Ok(base) => base,
                    Err(_) => dir_name,
                },
                None => dir_name,
            }
        } else {
            dir_name
        }
    };
    format!("{} {}", indentation, printable_name.display())
}

fn generate_bar(
    parent_bar: String,
    percent_size: f32,
) -> String {
    let num_bars = (parent_bar.chars().count() as f32 * percent_size) as usize;
    let mut num_not_my_bar = (parent_bar.chars().count() - num_bars) as i32;

    // recall darkest seen so far
    // while bright convert to darkest. if we have reached point then no conversion needed.
    let mut new_bar = "".to_string();
    let mut to_push = BLOCKS[4]; 

    for c in parent_bar.chars() {
        num_not_my_bar -= 1;
        if num_not_my_bar <= 0 {
            new_bar.push(BLOCKS[0]);
        }
        else if c == BLOCKS[0] {
            //push second darkest seen
            new_bar.push(to_push);
        }
        else if c == BLOCKS[4] {
            to_push = BLOCKS[3];
            new_bar.push(c);
        }
        else if c == BLOCKS[3] {
            to_push = BLOCKS[2];
            new_bar.push(c);
        }
        else if c == BLOCKS[2] {
            to_push = BLOCKS[1];
            new_bar.push(c);
        }
        else if c == BLOCKS[1] {
            new_bar.push(c);
        }
    }
    new_bar
}

// move inside display data?
pub fn format_string<P: AsRef<Path>>(
    dir_name: P,
    is_biggest: bool,
    display_data: &DisplayData,
    size: u64,
    indentation: &str,
    bar_text: &str,
) -> String {

    let pretty_size = format!("{:>5}", human_readable_number(size),);
    let percent_size = size as f32 / display_data.base_size as f32;
    let percent_size_str = format!("{:.0}%", percent_size * 100.0);

    let dir_name = dir_name.as_ref();
    let tree_and_path = get_printable_name(dir_name, display_data.short_paths, indentation);

    let printable_chars = tree_and_path.chars().count();
    let tree_and_path = tree_and_path
        + &(repeat(" ")
            .take(display_data.longest_string_length - printable_chars)
            .collect::<String>());

    format!(
        "{} {} │ {} │ {:>4}",
        if is_biggest && display_data.colors_on {
            Fixed(196).paint(pretty_size)
        } else {
            Style::new().paint(pretty_size)
        },
        tree_and_path,
        bar_text,
        percent_size_str,
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
