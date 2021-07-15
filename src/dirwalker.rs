use std::fs;

use crate::node::Node;
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;
use std::path::PathBuf;

use std::sync::atomic;
use std::sync::atomic::AtomicBool;

use std::collections::HashSet;

use crate::node::build_node;
use std::fs::DirEntry;

pub fn walk_it(
    dirs: HashSet<PathBuf>,
    ignore_directories: HashSet<PathBuf>,
    use_apparent_size: bool,
    by_filecount: bool,
    ignore_hidden: bool,
) -> (Vec<Node>, bool) {
    let permissions_flag = AtomicBool::new(false);

    let top_level_nodes: Vec<_> = dirs
        .into_iter()
        .filter_map(|d| {
            let n = walk(
                d,
                false,
                &permissions_flag,
                &ignore_directories,
                use_apparent_size,
                by_filecount,
                ignore_hidden,
            );
            match n {
                Some(n) => {
                    let mut inodes: HashSet<(u64, u64)> = HashSet::new();
                    clean_inodes(n, &mut inodes, use_apparent_size)
                }
                None => None,
            }
        })
        .collect();
    (top_level_nodes, permissions_flag.into_inner())
}

// Remove files which have the same inode, we don't want to double count them.
fn clean_inodes(
    x: Node,
    inodes: &mut HashSet<(u64, u64)>,
    use_apparent_size: bool,
) -> Option<Node> {
    if !use_apparent_size {
        if let Some(id) = x.inode_device {
            if inodes.contains(&id) {
                return None;
            }
            inodes.insert(id);
        }
    }

    let new_children: Vec<_> = x
        .children
        .into_iter()
        .filter_map(|c| clean_inodes(c, inodes, use_apparent_size))
        .collect();

    return Some(Node {
        name: x.name,
        size: x.size + new_children.iter().map(|c| c.size).sum::<u64>(),
        children: new_children,
        inode_device: x.inode_device,
    });
}

// todo: check for filesystem too
fn ignore_file(
    entry: &DirEntry,
    ignore_hidden: bool,
    ignore_directories: &HashSet<PathBuf>,
) -> bool {
    let is_dot_file = entry.file_name().to_str().unwrap_or("").starts_with('.');
    let is_ignored_path = ignore_directories.contains(&entry.path());
    (is_dot_file && ignore_hidden) || is_ignored_path
}

fn walk(
    dir: PathBuf,
    is_symlink: bool,
    permissions_flag: &AtomicBool,
    ignore_directories: &HashSet<PathBuf>,
    use_apparent_size: bool,
    by_filecount: bool,
    ignore_hidden: bool,
) -> Option<Node> {
    let mut children = vec![];

    if let Ok(entries) = fs::read_dir(dir.clone()) {
        children = entries
            .into_iter()
            .par_bridge()
            .filter_map(|entry| {
                if let Ok(ref entry) = entry {
                    // uncommenting the below line gives simpler code but
                    // rayon doesn't parallelise as well giving a 3X performance drop
                    // hence we unravel the recursion a bit

                    // return walk(entry.path(), permissions_flag, ignore_directories, use_apparent_size, by_filecount, ignore_hidden);

                    if !ignore_file(&entry, ignore_hidden, &ignore_directories) {
                        if let Ok(data) = entry.file_type() {
                            if data.is_dir() && !data.is_symlink() {
                                return walk(
                                    entry.path(),
                                    data.is_symlink(),
                                    permissions_flag,
                                    ignore_directories,
                                    use_apparent_size,
                                    by_filecount,
                                    ignore_hidden,
                                );
                            }
                            return build_node(
                                entry.path(),
                                vec![],
                                use_apparent_size,
                                data.is_symlink(),
                                by_filecount,
                            );
                        }
                    }
                } else {
                    permissions_flag.store(true, atomic::Ordering::Relaxed);
                }
                None
            })
            .collect();
    } else {
        permissions_flag.store(true, atomic::Ordering::Relaxed);
    }
    build_node(dir, children, use_apparent_size, is_symlink, by_filecount)
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(test)]
    fn create_node() -> Node {
        Node {
            name: PathBuf::new(),
            size: 10,
            children: vec![],
            inode_device: Some((5, 6)),
        }
    }

    #[test]
    fn test_should_ignore_file() {
        let mut inodes = HashSet::new();
        let n = create_node();

        // First time we insert the node
        assert!(clean_inodes(n.clone(), &mut inodes, false) == Some(n.clone()));

        // Second time is a duplicate - we ignore it
        assert!(clean_inodes(n.clone(), &mut inodes, false) == None);
    }

    #[test]
    fn test_should_not_ignore_files_if_using_apparent_size() {
        let mut inodes = HashSet::new();
        let n = create_node();

        // If using apparent size we include Nodes, even if duplicate inodes
        assert!(clean_inodes(n.clone(), &mut inodes, true) == Some(n.clone()));
        assert!(clean_inodes(n.clone(), &mut inodes, true) == Some(n.clone()));
    }
}
