use platform::get_metadata;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::DAY_SECONDS;

use crate::dir_walker::Operator;
use crate::platform;
use regex::Regex;

pub fn simplify_dir_names<P: AsRef<Path>>(dirs: &[P]) -> HashSet<PathBuf> {
    let mut top_level_names: HashSet<PathBuf> = HashSet::with_capacity(dirs.len());

    for t in dirs {
        let top_level_name = normalize_path(t);
        let mut can_add = true;
        let mut to_remove: Vec<PathBuf> = Vec::new();

        for tt in top_level_names.iter() {
            if is_a_parent_of(&top_level_name, tt) {
                to_remove.push(tt.to_path_buf());
            } else if is_a_parent_of(tt, &top_level_name) {
                can_add = false;
            }
        }
        for r in to_remove {
            top_level_names.remove(&r);
        }
        if can_add {
            top_level_names.insert(top_level_name);
        }
    }

    top_level_names
}

pub fn get_filesystem_devices<P: AsRef<Path>>(paths: &[P], follow_links: bool) -> HashSet<u64> {
    use std::fs;
    // Gets the device ids for the filesystems which are used by the argument paths
    paths
        .iter()
        .filter_map(|p| {
            let follow_links = if follow_links {
                // slow path: If dereference-links is set, then we check if the file is a symbolic link
                match fs::symlink_metadata(p) {
                    Ok(metadata) => metadata.file_type().is_symlink(),
                    Err(_) => false,
                }
            } else {
                false
            };
            match get_metadata(p, false, follow_links) {
                Some((_size, Some((_id, dev)), _time)) => Some(dev),
                _ => None,
            }
        })
        .collect()
}

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    // normalize path ...
    // 1. removing repeated separators
    // 2. removing interior '.' ("current directory") path segments
    // 3. removing trailing extra separators and '.' ("current directory") path segments
    // * `Path.components()` does all the above work; ref: <https://doc.rust-lang.org/std/path/struct.Path.html#method.components>
    // 4. changing to os preferred separator (automatically done by recollecting components back into a PathBuf)
    path.as_ref().components().collect()
}

pub fn is_filtered_out_due_to_regex(filter_regex: &[Regex], dir: &Path) -> bool {
    if filter_regex.is_empty() {
        false
    } else {
        filter_regex
            .iter()
            .all(|f| !f.is_match(&dir.as_os_str().to_string_lossy()))
    }
}

pub fn is_filtered_out_due_to_file_time(
    filter_time: &Option<(Operator, i64)>,
    actual_time: i64,
) -> bool {
    match filter_time {
        None => false,
        Some((Operator::Equal, bound_time)) => {
            !(actual_time >= *bound_time && actual_time < *bound_time + DAY_SECONDS)
        }
        Some((Operator::GreaterThan, bound_time)) => actual_time < *bound_time,
        Some((Operator::LessThan, bound_time)) => actual_time > *bound_time,
    }
}

pub fn is_filtered_out_due_to_invert_regex(filter_regex: &[Regex], dir: &Path) -> bool {
    filter_regex
        .iter()
        .any(|f| f.is_match(&dir.as_os_str().to_string_lossy()))
}

fn is_a_parent_of<P: AsRef<Path>>(parent: P, child: P) -> bool {
    let parent = parent.as_ref();
    let child = child.as_ref();
    child.starts_with(parent) && !parent.starts_with(child)
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_simplify_dir() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("a"));
        assert_eq!(simplify_dir_names(&["a"]), correct);
    }

    #[test]
    fn test_simplify_dir_rm_subdir() {
        let mut correct = HashSet::new();
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        assert_eq!(simplify_dir_names(&["a/b/c", "a/b", "a/b/d/f"]), correct);
        assert_eq!(simplify_dir_names(&["a/b", "a/b/c", "a/b/d/f"]), correct);
    }

    #[test]
    fn test_simplify_dir_duplicates() {
        let mut correct = HashSet::new();
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        correct.insert(PathBuf::from("c"));
        assert_eq!(
            simplify_dir_names(&[
                "a/b",
                "a/b//",
                "a/././b///",
                "c",
                "c/",
                "c/.",
                "c/././",
                "c/././."
            ]),
            correct
        );
    }
    #[test]
    fn test_simplify_dir_rm_subdir_and_not_substrings() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("b"));
        correct.insert(["c", "a", "b"].iter().collect::<PathBuf>());
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        assert_eq!(simplify_dir_names(&["a/b", "c/a/b/", "b"]), correct);
    }

    #[test]
    fn test_simplify_dir_dots() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("src"));
        assert_eq!(simplify_dir_names(&["src/."]), correct);
    }

    #[test]
    fn test_simplify_dir_substring_names() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("src"));
        correct.insert(PathBuf::from("src_v2"));
        assert_eq!(simplify_dir_names(&["src/", "src_v2"]), correct);
    }

    #[test]
    fn test_is_a_parent_of() {
        assert!(is_a_parent_of("/usr", "/usr/andy"));
        assert!(is_a_parent_of("/usr", "/usr/andy/i/am/descendant"));
        assert!(!is_a_parent_of("/usr", "/usr/."));
        assert!(!is_a_parent_of("/usr", "/usr/"));
        assert!(!is_a_parent_of("/usr", "/usr"));
        assert!(!is_a_parent_of("/usr/", "/usr"));
        assert!(!is_a_parent_of("/usr/andy", "/usr"));
        assert!(!is_a_parent_of("/usr/andy", "/usr/sibling"));
        assert!(!is_a_parent_of("/usr/folder", "/usr/folder_not_a_child"));
    }

    #[test]
    fn test_is_a_parent_of_root() {
        assert!(is_a_parent_of("/", "/usr/andy"));
        assert!(is_a_parent_of("/", "/usr"));
        assert!(!is_a_parent_of("/", "/"));
    }
}
