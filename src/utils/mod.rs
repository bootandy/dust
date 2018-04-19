use std::collections::HashMap;
use std::collections::HashSet;

use walkdir::WalkDir;

use std::path::Path;
use std::path::PathBuf;

mod platform;
use self::platform::*;

pub fn get_dir_tree(filenames: &Vec<&str>, apparent_size: bool) -> (bool, HashMap<String, u64>) {
    let mut permissions = 0;
    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
    let mut data: HashMap<String, u64> = HashMap::new();
    for b in filenames {
        examine_dir(
            &Path::new(b).to_path_buf(),
            apparent_size,
            &mut inodes,
            &mut data,
            &mut permissions,
        );
    }
    (permissions == 0, data)
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

pub fn find_big_ones<'a>(data: HashMap<String, u64>, max_to_show: usize) -> Vec<(String, u64)> {
    let mut new_l: Vec<(String, u64)> = data.iter().map(|(a, b)| (a.clone(), *b)).collect();
    new_l.sort_by(|a, b| b.1.cmp(&a.1));

    if new_l.len() > max_to_show {
        new_l[0..max_to_show + 1].to_vec()
    } else {
        new_l
    }
}
