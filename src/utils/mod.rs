use jwalk::DirEntry;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use jwalk::WalkDir;

mod platform;
use self::platform::*;

#[derive(Debug, Default, Eq)]
pub struct Node {
    pub name: PathBuf,
    pub size: u64,
    pub children: Vec<Node>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size == other.size {
            self.name.cmp(&other.name)
        } else {
            self.size.cmp(&other.size)
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.children == other.children
    }
}

pub fn is_a_parent_of<P: AsRef<Path>>(parent: P, child: P) -> bool {
    let parent = parent.as_ref();
    let child = child.as_ref();
    (child.starts_with(parent) && !parent.starts_with(child))
}

pub fn simplify_dir_names<P: AsRef<Path>>(filenames: Vec<P>) -> HashSet<PathBuf> {
    let mut top_level_names: HashSet<PathBuf> = HashSet::with_capacity(filenames.len());
    let mut to_remove: Vec<PathBuf> = Vec::with_capacity(filenames.len());

    for t in filenames {
        let top_level_name = normalize_path(t);
        let mut can_add = true;

        for tt in top_level_names.iter() {
            if is_a_parent_of(&top_level_name, tt) {
                to_remove.push(tt.to_path_buf());
            } else if is_a_parent_of(tt, &top_level_name) {
                can_add = false;
            }
        }
        to_remove.sort_unstable();
        top_level_names.retain(|tr| to_remove.binary_search(tr).is_err());
        to_remove.clear();
        if can_add {
            top_level_names.insert(top_level_name);
        }
    }

    top_level_names
}

pub fn get_dir_tree<P: AsRef<Path>>(
    top_level_names: &HashSet<P>,
    ignore_directories: &Option<Vec<PathBuf>>,
    apparent_size: bool,
    limit_filesystem: bool,
    threads: Option<usize>,
) -> (bool, HashMap<PathBuf, u64>) {
    let mut permissions = 0;
    let mut data: HashMap<PathBuf, u64> = HashMap::new();
    let restricted_filesystems = if limit_filesystem {
        get_allowed_filesystems(top_level_names)
    } else {
        None
    };

    for b in top_level_names.iter() {
        examine_dir(
            b,
            apparent_size,
            &restricted_filesystems,
            ignore_directories,
            &mut data,
            &mut permissions,
            threads,
        );
    }
    (permissions == 0, data)
}

fn get_allowed_filesystems<P: AsRef<Path>>(top_level_names: &HashSet<P>) -> Option<HashSet<u64>> {
    let mut limit_filesystems: HashSet<u64> = HashSet::new();
    for file_name in top_level_names.iter() {
        if let Ok(a) = get_filesystem(file_name) {
            limit_filesystems.insert(a);
        }
    }
    Some(limit_filesystems)
}

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    // normalize path ...
    // 1. removing repeated separators
    // 2. removing interior '.' ("current directory") path segments
    // 3. removing trailing extra separators and '.' ("current directory") path segments
    // * `Path.components()` does all the above work; ref: <https://doc.rust-lang.org/std/path/struct.Path.html#method.components>
    // 4. changing to os preferred separator (automatically done by recollecting components back into a PathBuf)
    path.as_ref().components().collect::<PathBuf>()
}

fn examine_dir<P: AsRef<Path>>(
    top_dir: P,
    apparent_size: bool,
    filesystems: &Option<HashSet<u64>>,
    ignore_directories: &Option<Vec<PathBuf>>,
    data: &mut HashMap<PathBuf, u64>,
    file_count_no_permission: &mut u64,
    threads: Option<usize>,
) {
    let top_dir = top_dir.as_ref();
    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
    let mut iter = WalkDir::new(top_dir)
        .preload_metadata(true)
        .skip_hidden(false);
    if let Some(threads_to_start) = threads {
        iter = iter.num_threads(threads_to_start);
    }
    'entry: for entry in iter {
        if let Ok(e) = entry {
            let maybe_size_and_inode = get_metadata(&e, apparent_size);
            if let Some(dirs) = ignore_directories {
                let path = e.path();
                let parts = path.components().collect::<Vec<std::path::Component>>();
                for d in dirs {
                    let seq = d.components().collect::<Vec<std::path::Component>>();
                    if parts
                        .windows(seq.len())
                        .any(|window| window.iter().collect::<PathBuf>() == *d)
                    {
                        continue 'entry;
                    }
                }
            }

            match maybe_size_and_inode {
                Some((size, maybe_inode)) => {
                    if !should_ignore_file(apparent_size, filesystems, &mut inodes, maybe_inode) {
                        process_file_with_size_and_inode(top_dir, data, e, size)
                    }
                }
                None => *file_count_no_permission += 1,
            }
        } else {
            *file_count_no_permission += 1
        }
    }
}

fn should_ignore_file(
    apparent_size: bool,
    restricted_filesystems: &Option<HashSet<u64>>,
    inodes: &mut HashSet<(u64, u64)>,
    maybe_inode: Option<(u64, u64)>,
) -> bool {
    if !apparent_size {
        if let Some(inode_dev_pair) = maybe_inode {
            // Ignore files on different devices (if flag applied)
            if restricted_filesystems.is_some()
                && !restricted_filesystems
                    .as_ref()
                    .unwrap()
                    .contains(&inode_dev_pair.1)
            {
                return true;
            }
            // Ignore files already visited or symlinked
            if inodes.contains(&inode_dev_pair) {
                return true;
            }
            inodes.insert(inode_dev_pair);
        }
    }
    false
}

fn process_file_with_size_and_inode<P: AsRef<Path>>(
    top_dir: P,
    data: &mut HashMap<PathBuf, u64>,
    e: DirEntry,
    size: u64,
) {
    let top_dir = top_dir.as_ref();
    // This path and all its parent paths have their counter incremented
    for path in e.path().ancestors() {
        // This is required due to bug in Jwalk that adds '/' to all sub dir lists
        // see: https://github.com/jessegrosjean/jwalk/issues/13
        if path.to_string_lossy() == "/" && top_dir.to_string_lossy() != "/" {
            continue;
        }
        let s = data.entry(normalize_path(path)).or_insert(0);
        *s += size;
        if path.starts_with(top_dir) && top_dir.starts_with(path) {
            break;
        }
    }
}

pub fn sort_by_size_first_name_second(a: &(PathBuf, u64), b: &(PathBuf, u64)) -> Ordering {
    let result = b.1.cmp(&a.1);
    if result == Ordering::Equal {
        a.0.cmp(&b.0)
    } else {
        result
    }
}

pub fn sort(data: HashMap<PathBuf, u64>) -> Vec<(PathBuf, u64)> {
    let mut new_l: Vec<(PathBuf, u64)> = data.iter().map(|(a, b)| (a.clone(), *b)).collect();
    new_l.sort_unstable_by(sort_by_size_first_name_second);
    new_l
}

pub fn find_big_ones(new_l: Vec<(PathBuf, u64)>, max_to_show: usize) -> Vec<(PathBuf, u64)> {
    if max_to_show > 0 && new_l.len() > max_to_show {
        new_l[0..max_to_show].to_vec()
    } else {
        new_l
    }
}

pub fn trim_deep_ones(
    input: Vec<(PathBuf, u64)>,
    max_depth: u64,
    top_level_names: &HashSet<PathBuf>,
) -> Vec<(PathBuf, u64)> {
    let mut result: Vec<(PathBuf, u64)> = Vec::with_capacity(input.len() * top_level_names.len());

    for name in top_level_names {
        let my_max_depth = name
            .components()
            .filter(|&c| match c {
                std::path::Component::Prefix(_) => false,
                _ => true,
            })
            .count()
            + max_depth as usize;

        for &(ref k, ref v) in input.iter() {
            if k.starts_with(name)
                && k.components()
                    .filter(|&c| match c {
                        std::path::Component::Prefix(_) => false,
                        _ => true,
                    })
                    .count()
                    <= my_max_depth
            {
                result.push((k.clone(), *v));
            }
        }
    }
    result
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_simplify_dir() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("a"));
        assert_eq!(simplify_dir_names(vec!["a"]), correct);
    }

    #[test]
    fn test_simplify_dir_rm_subdir() {
        let mut correct = HashSet::new();
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        assert_eq!(simplify_dir_names(vec!["a/b", "a/b/c", "a/b/d/f"]), correct);
    }

    #[test]
    fn test_simplify_dir_duplicates() {
        let mut correct = HashSet::new();
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        correct.insert(PathBuf::from("c"));
        assert_eq!(
            simplify_dir_names(vec![
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
        assert_eq!(simplify_dir_names(vec!["a/b", "c/a/b/", "b"]), correct);
    }

    #[test]
    fn test_simplify_dir_dots() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("src"));
        assert_eq!(simplify_dir_names(vec!["src/."]), correct);
    }

    #[test]
    fn test_simplify_dir_substring_names() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("src"));
        correct.insert(PathBuf::from("src_v2"));
        assert_eq!(simplify_dir_names(vec!["src/", "src_v2"]), correct);
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
