extern crate ansi_term;

use self::ansi_term::Colour::Fixed;
use self::ansi_term::Style;
use crate::utils::Node;

use terminal_size::{terminal_size, Height, Width};

use unicode_width::UnicodeWidthStr;

use std::cmp::max;
use std::cmp::min;
use std::iter::repeat;
use std::path::Path;

static UNITS: [char; 4] = ['T', 'G', 'M', 'K'];
static BLOCKS: [char; 5] = ['█', '▓', '▒', '░', ' '];
static DEFAULT_TERMINAL_WIDTH: u16 = 80;

pub struct DisplayData {
    pub short_paths: bool,
    pub is_reversed: bool,
    pub colors_on: bool,
    pub base_size: u64,
    pub longest_string_length: usize,
}

impl DisplayData {
    #[allow(clippy::collapsible_if)]
    fn get_tree_chars(&self, was_i_last: bool, has_children: bool) -> &'static str {
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

    fn is_biggest(&self, count: usize, max_siblings: u64) -> bool {
        if self.is_reversed {
            count == (max_siblings - 1) as usize
        } else {
            count == 0
        }
    }

    fn is_last(&self, count: usize, max_siblings: u64) -> bool {
        if self.is_reversed {
            count == 0
        } else {
            count == (max_siblings - 1) as usize
        }
    }
}

fn get_children_from_node(node: Node, is_reversed: bool) -> impl Iterator<Item = Node> {
    if is_reversed {
        let n: Vec<Node> = node.children.into_iter().rev().map(|a| a).collect();
        n.into_iter()
    } else {
        node.children.into_iter()
    }
}

struct DrawData<'a> {
    indent: String,
    percent_bar: String,
    display_data: &'a DisplayData,
}

impl DrawData<'_> {
    fn get_new_indent(&self, has_children: bool, was_i_last: bool) -> String {
        let chars = self.display_data.get_tree_chars(was_i_last, has_children);
        self.indent.to_string() + chars
    }

    fn percent_size(&self, node: &Node) -> f32 {
        node.size as f32 / self.display_data.base_size as f32
    }

    fn generate_bar(&self, node: &Node, level: usize) -> String {
        // temporary hack around rounding bug
        let mut num_bars =
            (self.percent_bar.chars().count() as f32 * self.percent_size(node)) as usize;
        if num_bars < self.percent_bar.chars().count() {
            num_bars = self.percent_bar.chars().count();
        }
        let mut num_not_my_bar = (self.percent_bar.chars().count() - num_bars) as i32;

        let mut new_bar = "".to_string();
        let idx = 5 - min(5, max(1, level));

        for c in self.percent_bar.chars() {
            num_not_my_bar -= 1;
            if num_not_my_bar <= 0 {
                new_bar.push(BLOCKS[0]);
            } else if c == BLOCKS[0] {
                new_bar.push(BLOCKS[idx]);
            } else {
                new_bar.push(c);
            }
        }
        new_bar
    }
}

fn get_width_of_terminal() -> u16 {
    // Windows CI runners detect a very low terminal width
    let default_width = {
        if let Some((Width(w), Height(_h))) = terminal_size() {
            w
        } else {
            0
        }
    };
    if default_width < DEFAULT_TERMINAL_WIDTH {
        DEFAULT_TERMINAL_WIDTH
    } else {
        default_width
    }
}

pub fn draw_it(
    permissions: bool,
    use_full_path: bool,
    is_reversed: bool,
    no_colors: bool,
    no_percents: bool,
    root_node: Node,
) {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }
    let mut longest_string_length = 0;
    for c in root_node.children.iter() {
        longest_string_length = max(
            longest_string_length,
            find_longest_dir_name(&c, "   ", !use_full_path),
        );
    }
    let terminal_width = get_width_of_terminal() - 16;

    let max_bar_length = if no_percents || longest_string_length >= terminal_width as usize {
        0
    } else {
        terminal_width as usize - longest_string_length
    };

    // handle usize error also add do not show fancy output option
    let bar_text = repeat(BLOCKS[0]).take(max_bar_length).collect::<String>();

    for c in get_children_from_node(root_node, is_reversed) {
        let display_data = DisplayData {
            short_paths: !use_full_path,
            is_reversed,
            colors_on: !no_colors,
            base_size: c.size,
            longest_string_length,
        };
        let draw_data = DrawData {
            indent: "".to_string(),
            percent_bar: bar_text.clone(),
            display_data: &display_data,
        };
        display_node(c, &draw_data, true, true);
    }
}

// can probably pass depth instead of indent down here.
fn find_longest_dir_name(node: &Node, indent: &str, long_paths: bool) -> usize {
    // Fix by calculating display width instead of number of chars
    //println!("{:?} {:?}", indent, node.name);
    let mut longest = UnicodeWidthStr::width(&*get_printable_name(&node.name, long_paths, indent));

    for c in node.children.iter() {
        // each tree drawing is 2 chars
        let full_indent: String = indent.to_string() + "  ";
        longest = max(longest, find_longest_dir_name(c, &*full_indent, long_paths));
    }
    longest
}

fn display_node(node: Node, draw_data: &DrawData, is_biggest: bool, is_last: bool) {
    let indent2 = draw_data.get_new_indent(!node.children.is_empty(), is_last);
    // hacky way of working out how deep we are in the tree
    let level = ((indent2.chars().count() - 1) / 2) - 1;
    let bar_text = draw_data.generate_bar(&node, level);

    let to_print = format_string(
        &node,
        &*indent2,
        &*bar_text,
        is_biggest,
        draw_data.display_data,
    );

    if !draw_data.display_data.is_reversed {
        println!("{}", to_print)
    }

    let dd = DrawData {
        indent: clean_indentation_string(&*indent2),
        percent_bar: bar_text,
        display_data: draw_data.display_data,
    };
    let num_siblings = node.children.len() as u64;

    for (count, c) in get_children_from_node(node, draw_data.display_data.is_reversed).enumerate() {
        let is_biggest = dd.display_data.is_biggest(count, num_siblings);
        let was_i_last = dd.display_data.is_last(count, num_siblings);
        display_node(c, &dd, is_biggest, was_i_last);
    }

    if draw_data.display_data.is_reversed {
        println!("{}", to_print)
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

fn get_printable_name<P: AsRef<Path>>(dir_name: &P, long_paths: bool, indentation: &str) -> String {
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

pub fn format_string(
    node: &Node,
    indent: &str,
    percent_bar: &str,
    is_biggest: bool,
    display_data: &DisplayData,
) -> String {
    let pretty_size = format!("{:>5}", human_readable_number(node.size));
    let percent_size = node.size as f32 / display_data.base_size as f32;
    let percent_size_str = format!("{:.0}%", percent_size * 100.0);

    let tree_and_path = get_printable_name(&node.name, display_data.short_paths, &*indent);

    let printable_chars = UnicodeWidthStr::width(&*tree_and_path);
    let tree_and_path = tree_and_path
        + &(repeat(" ")
            .take(display_data.longest_string_length - printable_chars)
            .collect::<String>());

    let percents = if percent_bar != "" {
        format!("│{} │ {:>4}", percent_bar, percent_size_str)
    } else {
        "".into()
    };

    format!(
        "{} {}{}",
        if is_biggest && display_data.colors_on {
            Fixed(196).paint(pretty_size)
        } else {
            Style::new().paint(pretty_size)
        },
        tree_and_path,
        percents,
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
