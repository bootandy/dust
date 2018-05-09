extern crate ansi_term;

use self::ansi_term::Colour::Fixed;
use std::collections::HashSet;

static UNITS: [char; 4] = ['T', 'G', 'M', 'K'];

pub fn draw_it(
    permissions: bool,
    short_paths: bool,
    depth: Option<u64>,
    base_dirs: HashSet<String>,
    to_display: Vec<(String, u64)>,
) -> () {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }
    let mut found = HashSet::new();

    for &(ref k, _) in to_display.iter() {
        if base_dirs.contains(k) {
            display_node(&k, &mut found, &to_display, true, short_paths, depth, "─┬")
        }
    }
}

fn get_size(nodes: &Vec<(String, u64)>, node_to_print: &String) -> Option<u64> {
    for &(ref k, ref v) in nodes.iter() {
        if *k == *node_to_print {
            return Some(*v);
        }
    }
    None
}

fn display_node<S: Into<String>>(
    node_to_print: &String,
    found: &mut HashSet<String>,
    to_display: &Vec<(String, u64)>,
    is_biggest: bool,
    short_paths: bool,
    depth: Option<u64>,
    indentation_str: S,
) {
    if found.contains(node_to_print) {
        return;
    }
    found.insert(node_to_print.to_string());

    let new_depth = match depth {
        None => None,
        Some(0) => return,
        Some(d) => Some(d - 1),
    };
    match get_size(to_display, node_to_print) {
        None => println!("Can not find path: {}", node_to_print),
        Some(size) => {
            let is = indentation_str.into();
            let ntp: &str = node_to_print.as_ref();

            print_this_node(ntp, size, is_biggest, short_paths, is.as_ref());
            let new_indent_str = clean_indentation_string(is);

            let num_slashes = node_to_print.matches('/').count();
            let mut num_siblings = count_siblings(to_display, num_slashes, ntp);

            let mut is_biggest = true;
            for &(ref k, _) in to_display.iter() {
                if k.starts_with(ntp) && k.matches('/').count() == num_slashes + 1 {
                    num_siblings -= 1;

                    let mut has_children = false;
                    if new_depth.is_none() || new_depth.unwrap() != 1 {
                        for &(ref k2, _) in to_display.iter() {
                            let kk: &str = k.as_ref();
                            if k2.starts_with(kk) && k2.matches('/').count() == num_slashes + 2 {
                                has_children = true;
                            }
                        }
                    }

                    display_node(
                        k,
                        found,
                        to_display,
                        is_biggest,
                        short_paths,
                        new_depth,
                        new_indent_str.to_string() + get_tree_chars(num_siblings, has_children),
                    );
                    is_biggest = false;
                }
            }
        }
    }
}

fn clean_indentation_string<S: Into<String>>(s: S) -> String {
    let mut is = s.into();
    is = is.replace("└─┬", "  ");
    is = is.replace("└──", "  ");
    is = is.replace("├──", "│ ");
    is = is.replace("├─┬", "│ ");
    is = is.replace("─┬", " ");
    is
}

fn count_siblings(to_display: &Vec<(String, u64)>, num_slashes: usize, ntp: &str) -> u64 {
    to_display.iter().fold(0, |a, b| {
        if b.0.starts_with(ntp) && b.0.matches('/').count() == num_slashes + 1 {
            a + 1
        } else {
            a
        }
    })
}

fn get_tree_chars(num_siblings: u64, has_children: bool) -> &'static str {
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
            Fixed(7).paint(size)
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
