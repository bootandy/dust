extern crate ansi_term;

use crate::utils::Node;

use self::ansi_term::Colour::Red;
use lscolors::{LsColors, Style};

use terminal_size::{terminal_size, Height, Width};

use unicode_width::UnicodeWidthStr;

use std::cmp::max;
use std::cmp::min;
use std::fs;
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
    pub ls_colors: LsColors,
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

    fn is_biggest(&self, num_siblings: usize, max_siblings: u64) -> bool {
        if self.is_reversed {
            num_siblings == (max_siblings - 1) as usize
        } else {
            num_siblings == 0
        }
    }

    fn is_last(&self, num_siblings: usize, max_siblings: u64) -> bool {
        if self.is_reversed {
            num_siblings == 0
        } else {
            num_siblings == (max_siblings - 1) as usize
        }
    }

    fn percent_size(&self, node: &Node) -> f32 {
        let result = node.size as f32 / self.base_size as f32;
        if result.is_normal() {
            result
        } else {
            0.0
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

    fn generate_bar(&self, node: &Node, level: usize) -> String {
        let chars_in_bar = self.percent_bar.chars().count();
        let num_bars = chars_in_bar as f32 * self.display_data.percent_size(node);
        let mut num_not_my_bar = (chars_in_bar as i32) - num_bars as i32;

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
    if let Some((Width(w), Height(_h))) = terminal_size() {
        max(w, DEFAULT_TERMINAL_WIDTH)
    } else {
        DEFAULT_TERMINAL_WIDTH
    }
}

pub fn draw_it(
    permissions: bool,
    use_full_path: bool,
    is_reversed: bool,
    no_colors: bool,
    no_percents: bool,
    by_filecount: bool,
    root_node: Node,
) {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }

    let longest_num = if by_filecount {
        human_readable_number(
            root_node.children.iter().map(|n| n.size).fold(0, max),
            by_filecount,
        )
        .len()
    } else {
        5
    };

    let longest_string_length = root_node
        .children
        .iter()
        .map(|c| find_longest_dir_name(&c, "   ", !use_full_path))
        .fold(0, max);

    let terminal_width = get_width_of_terminal() as usize - (9 + longest_num);

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
            ls_colors: LsColors::from_env().unwrap_or_default(),
        };
        let draw_data = DrawData {
            indent: "".to_string(),
            percent_bar: bar_text.clone(),
            display_data: &display_data,
        };
        display_node(c, &draw_data, longest_num, true, true, by_filecount);
    }
}

// We can probably pass depth instead of indent here.
// It is ugly to feed in '  ' instead of the actual tree characters but we don't need them yet.
fn find_longest_dir_name(node: &Node, indent: &str, long_paths: bool) -> usize {
    let printable_name = get_printable_name(&node.name, long_paths);
    let longest = get_unicode_width_of_indent_and_name(indent, &printable_name);

    // each none root tree drawing is 2 chars
    let full_indent: String = indent.to_string() + "  ";
    node.children
        .iter()
        .map(|c| find_longest_dir_name(c, &*full_indent, long_paths))
        .fold(longest, max)
}

fn display_node(
    node: Node,
    draw_data: &DrawData,
    longest_num: usize,
    is_biggest: bool,
    is_last: bool,
    display_count: bool,
) {
    let indent2 = draw_data.get_new_indent(!node.children.is_empty(), is_last);
    // hacky way of working out how deep we are in the tree
    let level = ((indent2.chars().count() - 1) / 2) - 1;
    let bar_text = draw_data.generate_bar(&node, level);

    let to_print = format_string(
        &node,
        longest_num,
        &*indent2,
        &*bar_text,
        is_biggest,
        display_count,
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
        display_node(c, &dd, longest_num, is_biggest, was_i_last, display_count);
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

fn get_printable_name<P: AsRef<Path>>(dir_name: &P, long_paths: bool) -> String {
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
    printable_name.display().to_string()
}

fn get_unicode_width_of_indent_and_name(indent: &str, name: &str) -> usize {
    let indent_and_name = format!("{} {}", indent, name);
    UnicodeWidthStr::width(&*indent_and_name)
}

pub fn format_string(
    node: &Node,
    longest_num: usize,
    indent: &str,
    percent_bar: &str,
    is_biggest: bool,
    display_count: bool,
    display_data: &DisplayData,
) -> String {
    let pretty_size = format!(
        "{0:>1$}",
        human_readable_number(node.size, display_count),
        longest_num,
    );

    let percent_size_str = format!("{:.0}%", display_data.percent_size(node) * 100.0);

    let name = get_printable_name(&node.name, display_data.short_paths);
    let width = get_unicode_width_of_indent_and_name(indent, &name);

    let (percents, name_and_padding) = if percent_bar != "" {
        let percents = format!("│{} │ {:>4}", percent_bar, percent_size_str);

        let name_and_padding = name
            + &(repeat(" ")
                .take(display_data.longest_string_length - width)
                .collect::<String>());
        (percents, name_and_padding)
    } else {
        ("".into(), name)
    };

    let pretty_size = if is_biggest && display_data.colors_on {
        format!("{}", Red.paint(pretty_size))
    } else {
        pretty_size
    };

    let pretty_name = if display_data.colors_on {
        let meta_result = fs::metadata(node.name.clone());
        let directory_color = display_data
            .ls_colors
            .style_for_path_with_metadata(node.name.clone(), meta_result.as_ref().ok());
        let ansi_style = directory_color
            .map(Style::to_ansi_term_style)
            .unwrap_or_default();
        format!("{}", ansi_style.paint(name_and_padding))
    } else {
        name_and_padding
    };

    format!("{} {} {}{}", pretty_size, indent, pretty_name, percents)
}

fn human_readable_number(size: u64, display_count: bool) -> String {
    if display_count {
        use thousands::Separable;
        size.separate_with_commas()
    } else {
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
        format!("{}B", size)
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_human_readable_number() {
        // Sizes
        assert_eq!(human_readable_number(1, false), "1B");
        assert_eq!(human_readable_number(956, false), "956B");
        assert_eq!(human_readable_number(1004, false), "1004B");
        assert_eq!(human_readable_number(1024, false), "1.0K");
        assert_eq!(human_readable_number(1536, false), "1.5K");
        assert_eq!(human_readable_number(1024 * 512, false), "512K");
        assert_eq!(human_readable_number(1024 * 1024, false), "1.0M");
        assert_eq!(
            human_readable_number(1024 * 1024 * 1024 - 1, false),
            "1023M"
        );
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 20, false), "20G");
        assert_eq!(
            human_readable_number(1024 * 1024 * 1024 * 1024, false),
            "1.0T"
        );

        // Counts
        assert_eq!(human_readable_number(1, true), "1");
        assert_eq!(human_readable_number(10, true), "10");
        assert_eq!(human_readable_number(100, true), "100");
        assert_eq!(human_readable_number(1_000, true), "1,000");
        assert_eq!(human_readable_number(10_000, true), "10,000");
        assert_eq!(human_readable_number(100_000, true), "100,000");
        assert_eq!(human_readable_number(1_000_000, true), "1,000,000");
    }
}
