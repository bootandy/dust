use std::collections::HashSet;

use std::fs;

use dust::Node;
use std::path::Path;

mod platform;
use self::platform::*;

pub fn get_dir_tree(filenames: &Vec<&str>, apparent_size: bool) -> (bool, Vec<Node>) {
    let mut permissions = true;
    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
    let mut results = vec![];
    for &b in filenames {
        let filename = strip_end_slashes(b);
        let (hp, data) = examine_dir(&Path::new(&filename), apparent_size, &mut inodes);
        permissions = permissions && hp;
        match data {
            Some(d) => results.push(d),
            None => permissions = false,
        }
    }
    (permissions, results)
}

fn strip_end_slashes(s: &str) -> String {
    let mut new_name = String::from(s);
    while new_name.chars().last() == Some('/') && new_name.len() != 1 {
        new_name.pop();
    }
    new_name
}

fn examine_dir(
    sdir: &Path,
    apparent_size: bool,
    inodes: &mut HashSet<(u64, u64)>,
) -> (bool, Option<Node>) {
    match fs::read_dir(sdir) {
        Ok(file_iter) => {
            let mut result = vec![];
            let mut have_permission = true;
            let mut total_size = 0;

            for single_path in file_iter {
                match single_path {
                    Ok(d) => {
                        let file_type = d.file_type().ok();
                        let maybe_size_and_inode = get_metadata(&d, apparent_size);

                        match (file_type, maybe_size_and_inode) {
                            (Some(file_type), Some((size, maybe_inode))) => {
                                if !apparent_size {
                                    if let Some(inode_dev_pair) = maybe_inode {
                                        if inodes.contains(&inode_dev_pair) {
                                            continue;
                                        }
                                        inodes.insert(inode_dev_pair);
                                    }
                                }
                                total_size += size;

                                if d.path().is_dir() && !file_type.is_symlink() {
                                    let (hp, child) = examine_dir(&d.path(), apparent_size, inodes);
                                    have_permission = have_permission && hp;

                                    match child {
                                        Some(c) => {
                                            total_size += c.size();
                                            result.push(c);
                                        }
                                        None => (),
                                    }
                                } else {
                                    let path_name = d.path().to_string_lossy().to_string();
                                    result.push(Node::new(path_name, size, vec![]))
                                }
                            }
                            (_, None) => have_permission = false,
                            (_, _) => (),
                        }
                    }
                    Err(_) => (),
                }
            }
            let n = Node::new(sdir.to_string_lossy().to_string(), total_size, result);
            (have_permission, Some(n))
        }
        Err(_) => (false, None),
    }
}

// We start with a list of root directories - these must be the biggest folders
// We then repeadedly merge in the children of the biggest directory - Each iteration
// the next biggest directory's children are merged in.
pub fn find_big_ones<'a>(l: &'a Vec<Node>, max_to_show: usize) -> Vec<&Node> {
    let mut new_l: Vec<&Node> = l.iter().map(|a| a).collect();
    new_l.sort();

    for processed_pointer in 0..max_to_show {
        if new_l.len() == processed_pointer {
            break;
        }
        // Must be a list of pointers into new_l otherwise b_list will go out of scope
        // when it is deallocated
        let mut b_list: Vec<&Node> = new_l[processed_pointer]
            .children()
            .iter()
            .map(|a| a)
            .collect();
        new_l.extend(b_list);
        new_l.sort();
    }

    if new_l.len() > max_to_show {
        new_l[0..max_to_show + 1].to_vec()
    } else {
        new_l
    }
}
