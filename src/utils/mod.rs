use std::collections::HashSet;

use std::fs::{self, ReadDir};
use std::io;

use dust::{DirEnt, Node};

mod platform;
use self::platform::*;

pub fn get_dir_tree(filenames: &Vec<&str>, apparent_size: bool) -> (bool, Vec<Node>) {
    let mut permissions = true;
    let mut results = vec![];
    for &b in filenames {
        let mut new_name = String::from(b);
        while new_name.chars().last() == Some('/') && new_name.len() != 1 {
            new_name.pop();
        }
        let (hp, data) = examine_dir_str(&new_name, apparent_size);
        permissions = permissions && hp;
        results.push(data);
    }
    (permissions, results)
}

fn examine_dir_str(loc: &str, apparent_size: bool) -> (bool, Node) {
    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
    let (hp, result) = examine_dir(fs::read_dir(loc), apparent_size, &mut inodes);

    // This needs to be folded into the below recursive call somehow
    let new_size = result.iter().fold(0, |a, b| a + b.entry().size());
    (hp, Node::new(DirEnt::new(loc, new_size), result))
}

fn examine_dir(
    a_dir: io::Result<ReadDir>,
    apparent_size: bool,
    inodes: &mut HashSet<(u64, u64)>,
) -> (bool, Vec<Node>) {
    let mut result = vec![];
    let mut have_permission = true;

    if a_dir.is_ok() {
        let paths = a_dir.unwrap();
        for dd in paths {
            match dd {
                Ok(d) => {
                    let file_type = d.file_type().ok();
                    let maybe_size_and_inode = get_metadata(&d, apparent_size);

                    match (file_type, maybe_size_and_inode) {
                        (Some(file_type), Some((size, inode))) => {
                            let s = d.path().to_string_lossy().to_string();
                            if !apparent_size {
                                if let Some(inode_dev_pair) = inode {
                                    if inodes.contains(&inode_dev_pair) {
                                        continue;
                                    }
                                    inodes.insert(inode_dev_pair);
                                }
                            }

                            if d.path().is_dir() && !file_type.is_symlink() {
                                let (hp, recursive) =
                                    examine_dir(fs::read_dir(d.path()), apparent_size, inodes);
                                have_permission = have_permission && hp;
                                let new_size =
                                    recursive.iter().fold(size, |a, b| a + b.entry().size());
                                result.push(Node::new(DirEnt::new(&s, new_size), recursive))
                            } else {
                                result.push(Node::new(DirEnt::new(&s, size), vec![]))
                            }
                        }
                        (_, None) => have_permission = false,
                        (_, _) => (),
                    }
                }
                Err(_) => (),
            }
        }
    } else {
        have_permission = false;
    }
    (have_permission, result)
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
