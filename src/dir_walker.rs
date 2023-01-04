use std::fs;

use crate::node::Node;
use crate::utils::is_filtered_out_due_to_invert_regex;
use crate::utils::is_filtered_out_due_to_regex;
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;
use regex::Regex;
use std::path::PathBuf;

use std::sync::atomic;
use std::sync::atomic::AtomicBool;

use std::collections::HashSet;

use crate::node::build_node;
use std::fs::DirEntry;

use crate::platform::get_metadata;

pub struct WalkData<'a> {
    pub ignore_directories: HashSet<PathBuf>,
    pub filter_regex: &'a [Regex],
    pub invert_filter_regex: &'a [Regex],
    pub allowed_filesystems: HashSet<u64>,
    pub use_apparent_size: bool,
    pub by_filecount: bool,
    pub ignore_hidden: bool,
}

pub fn walk_it(dirs: HashSet<PathBuf>, walk_data: WalkData) -> (Vec<Node>, bool) {
    let permissions_flag = AtomicBool::new(false);

    let mut inodes = HashSet::new();
    let top_level_nodes: Vec<_> = dirs
        .into_iter()
        .filter_map(|d| {
            clean_inodes(
                walk(d, &permissions_flag, &walk_data, 0)?,
                &mut inodes,
                walk_data.use_apparent_size,
            )
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
            if !inodes.insert(id) {
                return None;
            }
        }
    }

    // Sort Nodes so iteration order is predictable
    let mut tmp: Vec<_> = x.children;
    tmp.sort_by(sort_by_inode);
    let new_children: Vec<_> = tmp
        .into_iter()
        .filter_map(|c| clean_inodes(c, inodes, use_apparent_size))
        .collect();

    Some(Node {
        name: x.name,
        size: x.size + new_children.iter().map(|c| c.size).sum::<u64>(),
        children: new_children,
        inode_device: x.inode_device,
        depth: x.depth,
    })
}

fn sort_by_inode(a: &Node, b: &Node) -> std::cmp::Ordering {
    // Sorting by inode is quicker than by sorting by name/size
    if let Some(x) = a.inode_device {
        if let Some(y) = b.inode_device {
            if x.0 != y.0 {
                return x.0.cmp(&y.0);
            } else if x.1 != y.1 {
                return x.1.cmp(&y.1);
            }
        }
    }
    a.name.cmp(&b.name)
}

fn ignore_file(entry: &DirEntry, walk_data: &WalkData) -> bool {
    let is_dot_file = entry.file_name().to_str().unwrap_or("").starts_with('.');
    let is_ignored_path = walk_data.ignore_directories.contains(&entry.path());

    if !walk_data.allowed_filesystems.is_empty() {
        let size_inode_device = get_metadata(&entry.path(), false);

        if let Some((_size, Some((_id, dev)))) = size_inode_device {
            if !walk_data.allowed_filesystems.contains(&dev) {
                return true;
            }
        }
    }

    // Keeping `walk_data.filter_regex.is_empty()` is important for performance reasons, it stops unnecessary work
    if !walk_data.filter_regex.is_empty()
        && entry.path().is_file()
        && is_filtered_out_due_to_regex(walk_data.filter_regex, &entry.path())
    {
        return true;
    }

    if !walk_data.invert_filter_regex.is_empty()
        && entry.path().is_file()
        && is_filtered_out_due_to_invert_regex(walk_data.invert_filter_regex, &entry.path())
    {
        return true;
    }

    (is_dot_file && walk_data.ignore_hidden) || is_ignored_path
}

fn walk(
    dir: PathBuf,
    permissions_flag: &AtomicBool,
    walk_data: &WalkData,
    depth: usize,
) -> Option<Node> {
    let mut children = vec![];

    if let Ok(entries) = fs::read_dir(&dir) {
        children = entries
            .into_iter()
            .par_bridge()
            .filter_map(|entry| {
                if let Ok(ref entry) = entry {
                    // uncommenting the below line gives simpler code but
                    // rayon doesn't parallelize as well giving a 3X performance drop
                    // hence we unravel the recursion a bit

                    // return walk(entry.path(), permissions_flag, ignore_directories, allowed_filesystems, use_apparent_size, by_filecount, ignore_hidden);

                    if !ignore_file(entry, walk_data) {
                        if let Ok(data) = entry.file_type() {
                            return if data.is_dir() && !data.is_symlink() {
                                walk(entry.path(), permissions_flag, walk_data, depth + 1)
                            } else {
                                build_node(
                                    entry.path(),
                                    vec![],
                                    walk_data.filter_regex,
                                    walk_data.invert_filter_regex,
                                    walk_data.use_apparent_size,
                                    data.is_symlink(),
                                    data.is_file(),
                                    walk_data.by_filecount,
                                    depth,
                                )
                            };
                        }
                    }
                } else {
                    permissions_flag.store(true, atomic::Ordering::Relaxed);
                }
                None
            })
            .collect();
    } else {
        // Handle edge case where dust is called with a file instead of a directory
        if !dir.exists() {
            permissions_flag.store(true, atomic::Ordering::Relaxed);
        }
    }
    build_node(
        dir,
        children,
        walk_data.filter_regex,
        walk_data.invert_filter_regex,
        walk_data.use_apparent_size,
        false,
        false,
        walk_data.by_filecount,
        depth,
    )
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
            depth: 0,
        }
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_should_ignore_file() {
        let mut inodes = HashSet::new();
        let n = create_node();

        // First time we insert the node
        assert_eq!(clean_inodes(n.clone(), &mut inodes, false), Some(n.clone()));

        // Second time is a duplicate - we ignore it
        assert_eq!(clean_inodes(n.clone(), &mut inodes, false), None);
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_should_not_ignore_files_if_using_apparent_size() {
        let mut inodes = HashSet::new();
        let n = create_node();

        // If using apparent size we include Nodes, even if duplicate inodes
        assert_eq!(clean_inodes(n.clone(), &mut inodes, true), Some(n.clone()));
        assert_eq!(clean_inodes(n.clone(), &mut inodes, true), Some(n.clone()));
    }
}
