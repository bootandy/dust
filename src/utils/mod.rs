use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp::Ordering;

use walkdir::WalkDir;

use std::path::Path;
use std::path::PathBuf;

mod platform;
use self::platform::*;

pub fn get_dir_tree(
    filenames: &Vec<&str>,
    apparent_size: bool,
) -> (bool, HashMap<String, u64>, Vec<String>) {
    let mut permissions = 0;
    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
    let mut data: HashMap<String, u64> = HashMap::new();
    let mut top_level_names = Vec::new();

    for b in filenames {
        let top_level_name = strip_end_slashes(b);
        examine_dir(
            &Path::new(&top_level_name).to_path_buf(),
            apparent_size,
            &mut inodes,
            &mut data,
            &mut permissions,
        );
        top_level_names.push(top_level_name);
    }
    (permissions == 0, data, top_level_names)
}

fn strip_end_slashes(s: &str) -> String {
    let mut new_name = String::from(s);
    while new_name.chars().last() == Some('/') && new_name.len() != 1 {
        new_name.pop();
    }
    new_name
}

fn examine_dir(
    top_dir: &PathBuf,
    apparent_size: bool,
    inodes: &mut HashSet<(u64, u64)>,
    data: &mut HashMap<String, u64>,
    permissions: &mut u64,
) {
    for entry in WalkDir::new(top_dir) {
        match entry {
            Ok(e) => {
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
                            let s = data.entry(path_name).or_insert(0);
                            *s += size;
                            if e_path == *top_dir {
                                break;
                            }
                            e_path.pop();
                        }
                    }
                    None => *permissions += 1,
                }
            }
            _ => {}
        }
    }
}
pub fn compare_tuple(a :&(String, u64), b: &(String, u64)) -> Ordering {
    let result = b.1.cmp(&a.1);
    if result == Ordering::Equal {
        a.0.cmp(&b.0)
    } else {
        result
    }
}

pub fn sort<'a>(data: HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut new_l: Vec<(String, u64)> = data.iter().map(|(a, b)| (a.clone(), *b)).collect();
    new_l.sort_by(|a, b| compare_tuple(&a, &b));
    new_l
}

pub fn find_big_ones<'a>(new_l: Vec<(String, u64)>, max_to_show: usize) -> Vec<(String, u64)> {
    if max_to_show > 0 && new_l.len() > max_to_show {
        new_l[0..max_to_show + 1].to_vec()
    } else {
        new_l
    }
}
