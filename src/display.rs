use crate::display_node::DisplayNode;
use crate::node::FileTime;

use ansi_term::Colour::Red;
use lscolors::{LsColors, Style};

use unicode_width::UnicodeWidthStr;

use stfu8::encode_u8;

use chrono::{DateTime, Local, TimeZone, Utc};
use std::cmp::max;
use std::cmp::min;
use std::fs;
use std::iter::repeat_n;
use std::path::Path;
use thousands::Separable;

pub static UNITS: [char; 5] = ['P', 'T', 'G', 'M', 'K'];
static BLOCKS: [char; 5] = ['█', '▓', '▒', '░', ' '];
const FILETIME_SHOW_LENGTH: usize = 19;

pub struct InitialDisplayData {
    pub short_paths: bool,
    pub is_reversed: bool,
    pub colors_on: bool,
    pub by_filecount: bool,
    pub by_filetime: Option<FileTime>,
    pub is_screen_reader: bool,
    pub output_format: String,
    pub bars_on_right: bool,
}

pub struct DisplayData {
    pub initial: InitialDisplayData,
    pub num_chars_needed_on_left_most: usize,
    pub base_size: u64,
    pub longest_string_length: usize,
    pub ls_colors: LsColors,
}

impl DisplayData {
    fn get_tree_chars(&self, was_i_last: bool, has_children: bool) -> &'static str {
        match (self.initial.is_reversed, was_i_last, has_children) {
            (true, true, true) => "┌─┴",
            (true, true, false) => "┌──",
            (true, false, true) => "├─┴",
            (true, false, false) => "├──",
            (false, true, true) => "└─┬",
            (false, true, false) => "└──",
            (false, false, true) => "├─┬",
            (false, false, false) => "├──",
        }
    }

    fn is_biggest(&self, num_siblings: usize, max_siblings: u64) -> bool {
        if self.initial.is_reversed {
            num_siblings == (max_siblings - 1) as usize
        } else {
            num_siblings == 0
        }
    }

    fn is_last(&self, num_siblings: usize, max_siblings: u64) -> bool {
        if self.initial.is_reversed {
            num_siblings == 0
        } else {
            num_siblings == (max_siblings - 1) as usize
        }
    }

    fn percent_size(&self, node: &DisplayNode) -> f32 {
        let result = node.size as f32 / self.base_size as f32;
        if result.is_normal() { result } else { 0.0 }
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

    // TODO: can we test this?
    fn generate_bar(&self, node: &DisplayNode, level: usize) -> String {
        if self.display_data.initial.is_screen_reader {
            return level.to_string();
        }
        let chars_in_bar = self.percent_bar.chars().count();
        let num_bars = chars_in_bar as f32 * self.display_data.percent_size(node);
        let mut num_not_my_bar = (chars_in_bar as i32) - num_bars as i32;

        let mut new_bar = "".to_string();
        let idx = 5 - level.clamp(1, 4);

        let itr: Box<dyn Iterator<Item = char>> = if self.display_data.initial.bars_on_right {
            Box::new(self.percent_bar.chars())
        } else {
            Box::new(self.percent_bar.chars().rev())
        };

        for c in itr {
            num_not_my_bar -= 1;
            if num_not_my_bar <= 0 {
                new_bar.push(BLOCKS[0]);
            } else if c == BLOCKS[0] {
                new_bar.push(BLOCKS[idx]);
            } else {
                new_bar.push(c);
            }
        }
        if self.display_data.initial.bars_on_right {
            new_bar
        } else {
            new_bar.chars().rev().collect()
        }
    }
}

pub fn draw_it(
    idd: InitialDisplayData,
    root_node: &DisplayNode,
    no_percent_bars: bool,
    terminal_width: usize,
    skip_total: bool,
) {
    let num_chars_needed_on_left_most = if idd.by_filecount {
        let max_size = root_node.size;
        max_size.separate_with_commas().chars().count()
    } else if idd.by_filetime.is_some() {
        FILETIME_SHOW_LENGTH
    } else {
        find_biggest_size_str(root_node, &idd.output_format)
    };

    assert!(
        terminal_width > num_chars_needed_on_left_most + 2,
        "Not enough terminal width"
    );

    let allowed_width = terminal_width - num_chars_needed_on_left_most - 2;
    let num_indent_chars = 3;
    let longest_string_length =
        find_longest_dir_name(root_node, num_indent_chars, allowed_width, &idd);

    let max_bar_length = if no_percent_bars || longest_string_length + 7 >= allowed_width {
        0
    } else {
        allowed_width - longest_string_length - 7
    };

    let first_size_bar = repeat_n(BLOCKS[0], max_bar_length).collect();

    let display_data = DisplayData {
        initial: idd,
        num_chars_needed_on_left_most,
        base_size: root_node.size,
        longest_string_length,
        ls_colors: LsColors::from_env().unwrap_or_default(),
    };
    let draw_data = DrawData {
        indent: "".to_string(),
        percent_bar: first_size_bar,
        display_data: &display_data,
    };

    if !skip_total {
        display_node(root_node, &draw_data, true, true);
    } else {
        for (count, c) in root_node
            .get_children_from_node(draw_data.display_data.initial.is_reversed)
            .enumerate()
        {
            let is_biggest = display_data.is_biggest(count, root_node.num_siblings());
            let was_i_last = display_data.is_last(count, root_node.num_siblings());
            display_node(c, &draw_data, is_biggest, was_i_last);
        }
    }
}

fn find_biggest_size_str(node: &DisplayNode, output_format: &str) -> usize {
    let mut mx = human_readable_number(node.size, output_format)
        .chars()
        .count();
    for n in node.children.iter() {
        mx = max(mx, find_biggest_size_str(n, output_format));
    }
    mx
}

fn find_longest_dir_name(
    node: &DisplayNode,
    indent: usize,
    terminal: usize,
    idd: &InitialDisplayData,
) -> usize {
    let printable_name = get_printable_name(&node.name, idd.short_paths);

    let longest = if idd.is_screen_reader {
        UnicodeWidthStr::width(&*printable_name) + 1
    } else {
        min(
            UnicodeWidthStr::width(&*printable_name) + 1 + indent,
            terminal,
        )
    };

    // each none root tree drawing is 2 more chars, hence we increment indent by 2
    node.children
        .iter()
        .map(|c| find_longest_dir_name(c, indent + 2, terminal, idd))
        .fold(longest, max)
}

fn display_node(node: &DisplayNode, draw_data: &DrawData, is_biggest: bool, is_last: bool) {
    // hacky way of working out how deep we are in the tree
    let indent = draw_data.get_new_indent(!node.children.is_empty(), is_last);
    let level = ((indent.chars().count() - 1) / 2) - 1;
    let bar_text = draw_data.generate_bar(node, level);

    let to_print = format_string(node, &indent, &bar_text, is_biggest, draw_data.display_data);

    if !draw_data.display_data.initial.is_reversed {
        println!("{to_print}")
    }

    let dd = DrawData {
        indent: clean_indentation_string(&indent),
        percent_bar: bar_text,
        display_data: draw_data.display_data,
    };

    let num_siblings = node.num_siblings();

    for (count, c) in node
        .get_children_from_node(draw_data.display_data.initial.is_reversed)
        .enumerate()
    {
        let is_biggest = dd.display_data.is_biggest(count, num_siblings);
        let was_i_last = dd.display_data.is_last(count, num_siblings);
        display_node(c, &dd, is_biggest, was_i_last);
    }

    if draw_data.display_data.initial.is_reversed {
        println!("{to_print}")
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

pub fn get_printable_name<P: AsRef<Path>>(dir_name: &P, short_paths: bool) -> String {
    let dir_name = dir_name.as_ref();
    let printable_name = {
        if short_paths {
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
    encode_u8(printable_name.display().to_string().as_bytes())
}

fn pad_or_trim_filename(node: &DisplayNode, indent: &str, display_data: &DisplayData) -> String {
    let name = get_printable_name(&node.name, display_data.initial.short_paths);
    let indent_and_name = format!("{indent} {name}");
    let width = UnicodeWidthStr::width(&*indent_and_name);

    assert!(
        display_data.longest_string_length >= width,
        "Terminal width not wide enough to draw directory tree"
    );

    // Add spaces after the filename so we can draw the % used bar chart.
    name + " "
        .repeat(display_data.longest_string_length - width)
        .as_str()
}

fn maybe_trim_filename(name_in: String, indent: &str, display_data: &DisplayData) -> String {
    let indent_length = UnicodeWidthStr::width(indent);
    assert!(
        display_data.longest_string_length >= indent_length + 2,
        "Terminal width not wide enough to draw directory tree"
    );

    let max_size = display_data.longest_string_length - indent_length;
    if UnicodeWidthStr::width(&*name_in) > max_size {
        let name = name_in.chars().take(max_size - 2).collect::<String>();
        name + ".."
    } else {
        name_in
    }
}

pub fn format_string(
    node: &DisplayNode,
    indent: &str,
    bars: &str,
    is_biggest: bool,
    display_data: &DisplayData,
) -> String {
    let (percent, name_and_padding) = get_name_percent(node, indent, bars, display_data);
    let pretty_size = get_pretty_size(node, is_biggest, display_data);
    let pretty_name = get_pretty_name(node, name_and_padding, display_data);
    // we can clean this and the method below somehow, not sure yet
    if display_data.initial.is_screen_reader {
        // if screen_reader then bars is 'depth'
        format!("{pretty_name} {bars} {pretty_size}{percent}")
    } else if display_data.initial.by_filetime.is_some() {
        format!("{pretty_size} {indent}{pretty_name}")
    } else {
        format!("{pretty_size} {indent} {pretty_name}{percent}")
    }
}

fn get_name_percent(
    node: &DisplayNode,
    indent: &str,
    bar_chart: &str,
    display_data: &DisplayData,
) -> (String, String) {
    if display_data.initial.is_screen_reader {
        let percent = display_data.percent_size(node) * 100.0;
        let percent_size_str = format!("{percent:.0}%");
        let percents = format!(" {percent_size_str:>4}",);
        let name = pad_or_trim_filename(node, "", display_data);
        (percents, name)
    // Bar chart being empty may come from either config or the screen not being wide enough
    } else if !bar_chart.is_empty() {
        let percent = display_data.percent_size(node) * 100.0;
        let percent_size_str = format!("{percent:.0}%");
        let percents = format!("│{bar_chart} │ {percent_size_str:>4}");
        let name_and_padding = pad_or_trim_filename(node, indent, display_data);
        (percents, name_and_padding)
    } else {
        let n = get_printable_name(&node.name, display_data.initial.short_paths);
        let name = maybe_trim_filename(n, indent, display_data);
        ("".into(), name)
    }
}

fn get_pretty_size(node: &DisplayNode, is_biggest: bool, display_data: &DisplayData) -> String {
    let output = if display_data.initial.by_filecount {
        node.size.separate_with_commas()
    } else if display_data.initial.by_filetime.is_some() {
        get_pretty_file_modified_time(node.size as i64)
    } else {
        human_readable_number(node.size, &display_data.initial.output_format)
    };
    let spaces_to_add = display_data.num_chars_needed_on_left_most - output.chars().count();
    let output = " ".repeat(spaces_to_add) + output.as_str();

    if is_biggest && display_data.initial.colors_on {
        format!("{}", Red.paint(output))
    } else {
        output
    }
}

fn get_pretty_file_modified_time(timestamp: i64) -> String {
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();

    let local_datetime = datetime.with_timezone(&Local);

    local_datetime.format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn get_pretty_name(
    node: &DisplayNode,
    name_and_padding: String,
    display_data: &DisplayData,
) -> String {
    if display_data.initial.colors_on {
        let meta_result = fs::metadata(&node.name);
        let directory_color = display_data
            .ls_colors
            .style_for_path_with_metadata(&node.name, meta_result.as_ref().ok());
        let ansi_style = directory_color
            .map(Style::to_nu_ansi_term_style)
            .unwrap_or_default();
        let out = ansi_style.paint(name_and_padding);
        format!("{out}")
    } else {
        name_and_padding
    }
}

// If we are working with SI units or not
pub fn get_type_of_thousand(output_str: &str) -> u64 {
    if output_str.is_empty() {
        1024
    } else if output_str == "si" {
        1000
    } else if output_str.contains('i') || output_str.len() == 1 {
        1024
    } else {
        1000
    }
}

pub fn get_number_format(output_str: &str) -> Option<(u64, char)> {
    if output_str.starts_with('b') {
        return Some((1, 'B'));
    }
    for (i, u) in UNITS.iter().enumerate() {
        if output_str.starts_with((*u).to_ascii_lowercase()) {
            let marker = get_type_of_thousand(output_str).pow((UNITS.len() - i) as u32);
            return Some((marker, *u));
        }
    }
    None
}

pub fn human_readable_number(size: u64, output_str: &str) -> String {
    if output_str == "count" {
        return size.to_string();
    };
    match get_number_format(output_str) {
        Some((x, u)) => {
            format!("{}{}", (size / x), u)
        }
        None => {
            for (i, u) in UNITS.iter().enumerate() {
                let marker = get_type_of_thousand(output_str).pow((UNITS.len() - i) as u32);
                if size >= marker {
                    if size / marker < 10 {
                        return format!("{:.1}{}", (size as f32 / marker as f32), u);
                    } else {
                        return format!("{}{}", (size / marker), u);
                    }
                }
            }
            format!("{size}B")
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use std::path::PathBuf;

    #[cfg(test)]
    fn get_fake_display_data(longest_string_length: usize) -> DisplayData {
        let initial = InitialDisplayData {
            short_paths: true,
            is_reversed: false,
            colors_on: false,
            by_filecount: false,
            by_filetime: None,
            is_screen_reader: false,
            output_format: "".into(),
            bars_on_right: false,
        };
        DisplayData {
            initial,
            num_chars_needed_on_left_most: 5,
            base_size: 2_u64.pow(12), // 4.0K
            longest_string_length,
            ls_colors: LsColors::from_env().unwrap_or_default(),
        }
    }

    #[test]
    fn test_format_str() {
        let n = DisplayNode {
            name: PathBuf::from("/short"),
            size: 2_u64.pow(12), // This is 4.0K
            children: vec![],
        };
        let indent = "┌─┴";
        let percent_bar = "";
        let is_biggest = false;
        let data = get_fake_display_data(20);

        let s = format_string(&n, indent, percent_bar, is_biggest, &data);
        assert_eq!(s, " 4.0K ┌─┴ short");
    }

    #[test]
    fn test_format_str_long_name() {
        let name = "very_long_name_longer_than_the_eighty_character_limit_very_long_name_this_bit_will_truncate";
        let n = DisplayNode {
            name: PathBuf::from(name),
            size: 2_u64.pow(12), // This is 4.0K
            children: vec![],
        };
        let indent = "┌─┴";
        let percent_bar = "";
        let is_biggest = false;

        let data = get_fake_display_data(64);
        let s = format_string(&n, indent, percent_bar, is_biggest, &data);
        assert_eq!(
            s,
            " 4.0K ┌─┴ very_long_name_longer_than_the_eighty_character_limit_very_.."
        );
    }

    #[test]
    fn test_format_str_screen_reader() {
        let n = DisplayNode {
            name: PathBuf::from("/short"),
            size: 2_u64.pow(12), // This is 4.0K
            children: vec![],
        };
        let indent = "";
        let percent_bar = "3";
        let is_biggest = false;
        let mut data = get_fake_display_data(20);
        data.initial.is_screen_reader = true;

        let s = format_string(&n, indent, percent_bar, is_biggest, &data);
        assert_eq!(s, "short               3  4.0K 100%");
    }

    #[test]
    fn test_machine_readable_filecount() {
        assert_eq!(human_readable_number(1, "count"), "1");
        assert_eq!(human_readable_number(1000, "count"), "1000");
        assert_eq!(human_readable_number(1024, "count"), "1024");
    }

    #[test]
    fn test_human_readable_number() {
        assert_eq!(human_readable_number(1, ""), "1B");
        assert_eq!(human_readable_number(956, ""), "956B");
        assert_eq!(human_readable_number(1004, ""), "1004B");
        assert_eq!(human_readable_number(1024, ""), "1.0K");
        assert_eq!(human_readable_number(1536, ""), "1.5K");
        assert_eq!(human_readable_number(1024 * 512, ""), "512K");
        assert_eq!(human_readable_number(1024 * 1024, ""), "1.0M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 - 1, ""), "1023M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 20, ""), "20G");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 1024, ""), "1.0T");
        assert_eq!(
            human_readable_number(1024 * 1024 * 1024 * 1024 * 234, ""),
            "234T"
        );
        assert_eq!(
            human_readable_number(1024 * 1024 * 1024 * 1024 * 1024, ""),
            "1.0P"
        );
    }

    #[test]
    fn test_human_readable_number_si() {
        assert_eq!(human_readable_number(1024 * 100, ""), "100K");
        assert_eq!(human_readable_number(1024 * 100, "si"), "102K");
    }

    // Refer to https://en.wikipedia.org/wiki/Byte#Multiple-byte_units
    #[test]
    fn test_human_readable_number_kb() {
        let hrn = human_readable_number;
        assert_eq!(hrn(1023, "b"), "1023B");
        assert_eq!(hrn(1000 * 1000, "bytes"), "1000000B");
        assert_eq!(hrn(1023, "kb"), "1K");
        assert_eq!(hrn(1023, "k"), "0K");
        assert_eq!(hrn(1023, "kib"), "0K");
        assert_eq!(hrn(1024, "kib"), "1K");
        assert_eq!(hrn(1024 * 512, "kib"), "512K");
        assert_eq!(hrn(1024 * 1024, "kib"), "1024K");
        assert_eq!(hrn(1024 * 1000 * 1000 * 20, "kib"), "20000000K");
        assert_eq!(hrn(1024 * 1024 * 1000 * 20, "mib"), "20000M");
        assert_eq!(hrn(1024 * 1024 * 1024 * 20, "gib"), "20G");
    }

    #[cfg(test)]
    fn build_draw_data(disp: &DisplayData, size: u32) -> (DrawData<'_>, DisplayNode) {
        let n = DisplayNode {
            name: PathBuf::from("/short"),
            size: 2_u64.pow(size),
            children: vec![],
        };
        let first_size_bar = repeat_n(BLOCKS[0], 13).collect();
        let dd = DrawData {
            indent: "".into(),
            percent_bar: first_size_bar,
            display_data: disp,
        };
        (dd, n)
    }

    #[test]
    fn test_draw_data() {
        let disp = &get_fake_display_data(20);
        let (dd, n) = build_draw_data(disp, 12);
        let bar = dd.generate_bar(&n, 1);
        assert_eq!(bar, "█████████████");
    }

    #[test]
    fn test_draw_data2() {
        let disp = &get_fake_display_data(20);
        let (dd, n) = build_draw_data(disp, 11);
        let bar = dd.generate_bar(&n, 2);
        assert_eq!(bar, "███████░░░░░░");
    }
    #[test]
    fn test_draw_data3() {
        let mut disp = get_fake_display_data(20);
        let (dd, n) = build_draw_data(&disp, 11);
        let bar = dd.generate_bar(&n, 3);
        assert_eq!(bar, "███████▒▒▒▒▒▒");

        disp.initial.bars_on_right = true;
        let (dd, n) = build_draw_data(&disp, 11);
        let bar = dd.generate_bar(&n, 3);
        assert_eq!(bar, "▒▒▒▒▒▒███████")
    }
    #[test]
    fn test_draw_data4() {
        let disp = &get_fake_display_data(20);
        let (dd, n) = build_draw_data(disp, 10);
        // After 4 we have no more levels of shading so 4+ is the same
        let bar = dd.generate_bar(&n, 4);
        assert_eq!(bar, "████▓▓▓▓▓▓▓▓▓");
        let bar = dd.generate_bar(&n, 5);
        assert_eq!(bar, "████▓▓▓▓▓▓▓▓▓");
    }

    #[test]
    fn test_get_pretty_file_modified_time() {
        // Create a timestamp for 2023-07-12 00:00:00 in local time
        let local_dt = Local.with_ymd_and_hms(2023, 7, 12, 0, 0, 0).unwrap();
        let timestamp = local_dt.timestamp();

        // Format expected output
        let expected_output = local_dt.format("%Y-%m-%dT%H:%M:%S").to_string();

        assert_eq!(get_pretty_file_modified_time(timestamp), expected_output);

        // Test another timestamp
        let local_dt = Local.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap();
        let timestamp = local_dt.timestamp();
        let expected_output = local_dt.format("%Y-%m-%dT%H:%M:%S").to_string();

        assert_eq!(get_pretty_file_modified_time(timestamp), expected_output);

        // Test timestamp for epoch start (1970-01-01T00:00:00)
        let local_dt = Local.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
        let timestamp = local_dt.timestamp();
        let expected_output = local_dt.format("%Y-%m-%dT%H:%M:%S").to_string();

        assert_eq!(get_pretty_file_modified_time(timestamp), expected_output);

        // Test a future timestamp
        let local_dt = Local.with_ymd_and_hms(2030, 12, 25, 6, 30, 0).unwrap();
        let timestamp = local_dt.timestamp();
        let expected_output = local_dt.format("%Y-%m-%dT%H:%M:%S").to_string();

        assert_eq!(get_pretty_file_modified_time(timestamp), expected_output);
    }
}
