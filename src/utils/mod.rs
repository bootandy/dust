use jwalk::DirEntry;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

use jwalk::WalkDir;

mod platform;
use self::platform::*;

#[derive(Debug, Default, Eq)]
pub struct Node {
    pub name: String,
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

pub fn is_a_parent_of(parent: &str, child: &str) -> bool {
    (child.starts_with(parent) && child.chars().nth(parent.chars().count()) == Some('/'))
        || parent == "/"
}

pub fn simplify_dir_names(filenames: Vec<&str>) -> HashSet<String> {
    let mut top_level_names: HashSet<String> = HashSet::with_capacity(filenames.len());
    let mut to_remove: Vec<String> = Vec::with_capacity(filenames.len());

    for t in filenames {
        let top_level_name = strip_end_slash(t);
        let mut can_add = true;

        for tt in top_level_names.iter() {
            if is_a_parent_of(&top_level_name, tt) {
                to_remove.push(tt.to_string());
            } else if is_a_parent_of(tt, &top_level_name) {
                can_add = false;
            }
        }
        to_remove.sort_unstable();
        top_level_names.retain(|tr| to_remove.binary_search(tr).is_err());
        to_remove.clear();
        if can_add {
            top_level_names.insert(strip_end_slash(t).to_owned());
        }
    }

    top_level_names
}

pub fn get_dir_tree(
    top_level_names: &HashSet<String>,
    ignore_directories: Option<Vec<&str>>,
    apparent_size: bool,
    limit_filesystem: bool,
    threads: Option<usize>,
) -> (bool, HashMap<String, u64>) {
    let mut permissions = 0;
    let mut data: HashMap<String, u64> = HashMap::new();
    let restricted_filesystems = if limit_filesystem {
        get_allowed_filesystems(top_level_names)
    } else {
        None
    };

    for b in top_level_names.iter() {
        examine_dir(
            &b,
            apparent_size,
            &restricted_filesystems,
            &ignore_directories,
            &mut data,
            &mut permissions,
            threads,
        );
    }
    (permissions == 0, data)
}

fn get_allowed_filesystems(top_level_names: &HashSet<String>) -> Option<HashSet<u64>> {
    let mut limit_filesystems: HashSet<u64> = HashSet::new();
    for file_name in top_level_names.iter() {
        if let Ok(a) = get_filesystem(file_name) {
            limit_filesystems.insert(a);
        }
    }
    Some(limit_filesystems)
}

pub fn strip_end_slash(mut new_name: &str) -> &str {
    while (new_name.ends_with('/') || new_name.ends_with("/.")) && new_name.len() > 1 {
        new_name = &new_name[..new_name.len() - 1];
    }
    new_name
}

fn examine_dir(
    top_dir: &str,
    apparent_size: bool,
    filesystems: &Option<HashSet<u64>>,
    ignore_directories: &Option<Vec<&str>>,
    data: &mut HashMap<String, u64>,
    file_count_no_permission: &mut u64,
    threads: Option<usize>,
) {
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
            if let Some(d) = ignore_directories {
                for s in d {
                    if e.path().to_string_lossy().contains(*s) {
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

fn process_file_with_size_and_inode(
    top_dir: &str,
    data: &mut HashMap<String, u64>,
    e: DirEntry,
    size: u64,
) {
    // This path and all its parent paths have their counter incremented
    for path_name in e.path().ancestors() {
        // This is required due to bug in Jwalk that adds '/' to all sub dir lists
        // see: https://github.com/jessegrosjean/jwalk/issues/13
        if path_name.to_string_lossy() == "/" && top_dir != "/" {
            continue;
        }
        let path_name = path_name.to_string_lossy();
        let s = data.entry(path_name.to_string()).or_insert(0);
        *s += size;
        if path_name == top_dir {
            break;
        }
    }
}

pub fn sort_by_size_first_name_second(a: &(String, u64), b: &(String, u64)) -> Ordering {
    let result = b.1.cmp(&a.1);
    if result == Ordering::Equal {
        a.0.cmp(&b.0)
    } else {
        result
    }
}

pub fn sort(data: HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut new_l: Vec<(String, u64)> = data.iter().map(|(a, b)| (a.clone(), *b)).collect();
    new_l.sort_unstable_by(sort_by_size_first_name_second);
    new_l
}

pub fn find_big_ones(new_l: Vec<(String, u64)>, max_to_show: usize) -> Vec<(String, u64)> {
    if max_to_show > 0 && new_l.len() > max_to_show {
        new_l[0..max_to_show].to_vec()
    } else {
        new_l
    }
}

pub fn trim_deep_ones(
    input: Vec<(String, u64)>,
    max_depth: u64,
    top_level_names: &HashSet<String>,
) -> Vec<(String, u64)> {
    let mut result: Vec<(String, u64)> = Vec::with_capacity(input.len() * top_level_names.len());

    for name in top_level_names {
        let my_max_depth = name.matches('/').count() + max_depth as usize;
        let name_ref: &str = name.as_ref();

        for &(ref k, ref v) in input.iter() {
            if k.starts_with(name_ref) && k.matches('/').count() <= my_max_depth {
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
        correct.insert("a".to_string());
        assert_eq!(simplify_dir_names(vec!["a"]), correct);
    }

    #[test]
    fn test_simplify_dir_rm_subdir() {
        let mut correct = HashSet::new();
        correct.insert("a/b".to_string());
        assert_eq!(simplify_dir_names(vec!["a/b", "a/b/c", "a/b/d/f"]), correct);
    }

    #[test]
    fn test_simplify_dir_duplicates() {
        let mut correct = HashSet::new();
        correct.insert("a/b".to_string());
        correct.insert("c".to_string());
        assert_eq!(simplify_dir_names(vec!["a/b", "a/b//", "c", "c/"]), correct);
    }
    #[test]
    fn test_simplify_dir_rm_subdir_and_not_substrings() {
        let mut correct = HashSet::new();
        correct.insert("b".to_string());
        correct.insert("c/a/b".to_string());
        correct.insert("a/b".to_string());
        assert_eq!(simplify_dir_names(vec!["a/b", "c/a/b/", "b"]), correct);
    }

    #[test]
    fn test_simplify_dir_dots() {
        let mut correct = HashSet::new();
        correct.insert("src".to_string());
        assert_eq!(simplify_dir_names(vec!["src/."]), correct);
    }

    #[test]
    fn test_simplify_dir_substring_names() {
        let mut correct = HashSet::new();
        correct.insert("src".to_string());
        correct.insert("src_v2".to_string());
        assert_eq!(simplify_dir_names(vec!["src/", "src_v2"]), correct);
    }

    #[test]
    fn test_is_a_parent_of() {
        assert!(is_a_parent_of("/usr", "/usr/andy"));
        assert!(is_a_parent_of("/usr", "/usr/andy/i/am/descendant"));
        assert!(!is_a_parent_of("/usr/andy", "/usr"));
        assert!(!is_a_parent_of("/usr/andy", "/usr/sibling"));
    }

    #[test]
    fn test_is_a_parent_of_root() {
        assert!(is_a_parent_of("/", "/usr/andy"));
        assert!(is_a_parent_of("/", "/usr"));
    }
}
