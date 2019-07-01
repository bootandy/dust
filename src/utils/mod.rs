use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

use walkdir::WalkDir;

mod platform;
use self::platform::*;

pub fn simplify_dir_names(filenames: Vec<&str>) -> HashSet<String> {
    let mut top_level_names: HashSet<String> = HashSet::new();

    for t in filenames {
        let top_level_name = strip_end_slashes(t);
        let mut can_add = true;
        let mut to_remove: Vec<String> = Vec::new();

        for tt in top_level_names.iter() {
            let temp = tt.to_string();
            if top_level_name.starts_with(&temp) {
                can_add = false;
            } else if tt.starts_with(&top_level_name) {
                to_remove.push(temp);
            }
        }
        for tr in to_remove {
            top_level_names.remove(&tr);
        }
        if can_add {
            top_level_names.insert(top_level_name);
        }
    }

    top_level_names
}

pub fn get_dir_tree(
    top_level_names: &HashSet<String>,
    apparent_size: bool,
) -> (bool, HashMap<String, u64>) {
    let mut permissions = 0;
    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
    let mut data: HashMap<String, u64> = HashMap::new();

    for b in top_level_names.iter() {
        examine_dir(&b, apparent_size, &mut inodes, &mut data, &mut permissions);
    }
    (permissions == 0, data)
}

fn strip_end_slashes(s: &str) -> String {
    let mut new_name = String::from(s);
    while (new_name.ends_with('/') || new_name.ends_with("/.")) && new_name.len() != 1 {
        new_name.pop();
    }
    new_name
}

fn examine_dir(
    top_dir: &str,
    apparent_size: bool,
    inodes: &mut HashSet<(u64, u64)>,
    data: &mut HashMap<String, u64>,
    permissions: &mut u64,
) {
    for entry in WalkDir::new(top_dir) {
        if let Ok(e) = entry {
            let maybe_size_and_inode = get_metadata(&e, apparent_size);

            match maybe_size_and_inode {
                Some((size, maybe_inode)) => {
                    if !apparent_size {
                        if let Some(inode_dev_pair) = maybe_inode {
                            if inodes.contains(&inode_dev_pair) {
                                continue;
                            }
                            inodes.insert(inode_dev_pair);
                        }
                    }
                    let mut e_path = e.path().to_path_buf();
                    loop {
                        let path_name = e_path.to_string_lossy().to_string();
                        let s = data.entry(path_name.clone()).or_insert(0);
                        *s += size;
                        if path_name == *top_dir {
                            break;
                        }
                        assert!(path_name != "");
                        e_path.pop();
                    }
                }
                None => *permissions += 1,
            }
        }
    }
}
pub fn compare_tuple(a: &(String, u64), b: &(String, u64)) -> Ordering {
    let result = b.1.cmp(&a.1);
    if result == Ordering::Equal {
        a.0.cmp(&b.0)
    } else {
        result
    }
}

pub fn sort(data: HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut new_l: Vec<(String, u64)> = data.iter().map(|(a, b)| (a.clone(), *b)).collect();
    new_l.sort_by(|a, b| compare_tuple(&a, &b));
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
    let mut result: Vec<(String, u64)> = vec![];

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
        correct.insert("a/b".to_string());
        correct.insert("c/a/b".to_string());
        correct.insert("b".to_string());
        assert_eq!(simplify_dir_names(vec!["a/b", "c/a/b/", "b"]), correct);
    }

    #[test]
    fn test_simplify_dir_dots() {
        let mut correct = HashSet::new();
        correct.insert("src".to_string());
        assert_eq!(simplify_dir_names(vec!["src/."]), correct);
    }
}
