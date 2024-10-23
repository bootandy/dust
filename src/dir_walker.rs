use std::cmp::Ordering;
use std::fs;
use std::fs::Metadata;
use std::os::linux::fs::MetadataExt;
use std::sync::Arc;
use std::sync::Mutex;

use crate::node::Node;
use crate::platform::InodeAndDevice;
use crate::progress::Operation;
use crate::progress::PAtomicInfo;
use crate::progress::RuntimeErrors;
use crate::progress::ORDERING;
use crate::utils::is_filtered_out_due_to_file_time;
use crate::utils::is_filtered_out_due_to_invert_regex;
use crate::utils::is_filtered_out_due_to_regex;
use procfs::process::FDTarget;
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;
use regex::Regex;
use std::path::PathBuf;

use std::collections::HashSet;

use crate::node::build_node;
use std::fs::DirEntry;

use crate::node::FileTime;
use crate::platform::get_metadata;

#[derive(Debug)]
pub enum Operater {
    Equal = 0,
    LessThan = 1,
    GreaterThan = 2,
}

pub struct WalkData<'a> {
    pub ignore_directories: HashSet<PathBuf>,
    pub filter_regex: &'a [Regex],
    pub invert_filter_regex: &'a [Regex],
    pub allowed_filesystems: HashSet<u64>,
    pub filter_modified_time: Option<(Operater, i64)>,
    pub filter_accessed_time: Option<(Operater, i64)>,
    pub filter_changed_time: Option<(Operater, i64)>,
    pub use_apparent_size: bool,
    pub by_filecount: bool,
    pub by_filetime: &'a Option<FileTime>,
    pub ignore_hidden: bool,
    pub follow_links: bool,
    pub progress_data: Arc<PAtomicInfo>,
    pub errors: Arc<Mutex<RuntimeErrors>>,
}

/// Return deleted file still accessed by a process by walking /proc/$PID/fd/$FD
/// Deleted files have nlinks == 0
fn get_deleted_files() -> Vec<(PathBuf, Metadata)> {
    let mut deleted_files = Vec::new();

    for p in procfs::process::all_processes().unwrap() {
        let Ok(p) = p else {
            continue;
        };
        let Ok(fds) = p.fd() else {
            continue;
        };

        for fd in fds {
            let Ok(fd) = fd else {
                continue;
            };

            if let FDTarget::Path(path) = &fd.target {
                let proc_fd = format!("/proc/{}/fd/{}", p.pid, fd.fd);
                let Ok(metadata) = std::fs::metadata(&proc_fd) else {
                    continue;
                };

                if metadata.st_nlink() == 0 {
                    // TODO: remove " (deleted)", not part of actual name
                    deleted_files.push((path.clone(), metadata));
                }
            }
        }
    }

    deleted_files
}

pub fn walk_it(dirs: HashSet<PathBuf>, walk_data: &WalkData) -> Vec<Node> {
    let mut inodes = HashSet::new();
    let mut top_level_nodes: Vec<_> = dirs
        .into_iter()
        .filter_map(|d| {
            let prog_data = &walk_data.progress_data;
            prog_data.clear_state(&d);
            let node = walk(d, walk_data, 0)?;

            prog_data.state.store(Operation::PREPARING, ORDERING);

            clean_inodes(node, &mut inodes, walk_data)
        })
        .collect();

    // TODO: use a flag
    let handle_deleted_files = true;

    if handle_deleted_files {
        let deleted_files: Vec<_> = get_deleted_files()
            .into_iter()
            .filter(|(_path, metadata)| {
                let inode_and_device = (metadata.st_ino(), metadata.st_dev());
                // ignore inodes already collected as part of regular files
                !inodes.contains(&inode_and_device)
            })
            .collect();

        // we try to insert deleted files in the node tree
        for (path, m) in &deleted_files {
            for mut top_level_node in &mut top_level_nodes {
                // deleted files are always absolute, need to canonicalize the node tree
                let absolute_path = top_level_node.name.canonicalize().unwrap();
                if path.starts_with(absolute_path.components().next().unwrap()) {
                    let inserted = insert_deleted_file_in_node(
                        path.clone(),
                        m,
                        &mut top_level_node,
                        &walk_data,
                        0,
                    );

                    if !inserted {
                        println!("Couldn't insert {:?}", &path);
                    }
                }
            }
        }
    }

    top_level_nodes
}

/// try to insert `path` in `root`, or its children
/// `path` is absolute
fn insert_deleted_file_in_node(
    path: PathBuf,
    m: &Metadata,
    root: &mut Node,
    walk_data: &WalkData,
    depth: usize,
) -> bool {
    if path.parent().unwrap() == &root.name.canonicalize().unwrap() {
        // we found the node that represents the parent dir
        // TODO: filecount, filetime, regex...
        let size = if walk_data.use_apparent_size {
            m.st_size()
        } else {
            m.st_blocks() * 512
        };

        root.size += size;

        let node = Node {
            name: path.clone(),
            size,
            children: vec![],
            inode_device: Some((m.st_ino(), m.st_dev())),
            depth,
        };

        root.children.push(node);
        println!("inserted {:?} in {:?}", path, root.name);

        return true;
    }

    for child in &mut root.children {
        if path.starts_with(&child.name.canonicalize().unwrap()) {
            return insert_deleted_file_in_node(path, m, child, &walk_data, depth + 1);
        }
    }
    return false;
}

// Remove files which have the same inode, we don't want to double count them.
fn clean_inodes(
    x: Node,
    inodes: &mut HashSet<InodeAndDevice>,
    walk_data: &WalkData,
) -> Option<Node> {
    if !walk_data.use_apparent_size {
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
        .filter_map(|c| clean_inodes(c, inodes, walk_data))
        .collect();

    let actual_size = if walk_data.by_filetime.is_some() {
        // If by_filetime is Some, directory 'size' is the maximum filetime among child files instead of disk size
        new_children
            .iter()
            .map(|c| c.size)
            .chain(std::iter::once(x.size))
            .max()
            .unwrap_or(0)
    } else {
        // If by_filetime is None, directory 'size' is the sum of disk sizes or file counts of child files
        x.size + new_children.iter().map(|c| c.size).sum::<u64>()
    };

    Some(Node {
        name: x.name,
        size: actual_size,
        children: new_children,
        inode_device: x.inode_device,
        depth: x.depth,
    })
}

fn sort_by_inode(a: &Node, b: &Node) -> std::cmp::Ordering {
    // Sorting by inode is quicker than by sorting by name/size
    match (a.inode_device, b.inode_device) {
        (Some(x), Some(y)) => {
            if x.0 != y.0 {
                x.0.cmp(&y.0)
            } else if x.1 != y.1 {
                x.1.cmp(&y.1)
            } else {
                a.name.cmp(&b.name)
            }
        }
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => a.name.cmp(&b.name),
    }
}

fn ignore_file(entry: &DirEntry, walk_data: &WalkData) -> bool {
    let is_dot_file = entry.file_name().to_str().unwrap_or("").starts_with('.');
    let is_ignored_path = walk_data.ignore_directories.contains(&entry.path());
    let follow_links =
        walk_data.follow_links && entry.file_type().map_or(false, |ft| ft.is_symlink());

    if !walk_data.allowed_filesystems.is_empty() {
        let size_inode_device = get_metadata(entry.path(), false, follow_links);
        if let Some((_size, Some((_id, dev)), _gunk)) = size_inode_device {
            if !walk_data.allowed_filesystems.contains(&dev) {
                return true;
            }
        }
    }
    if walk_data.filter_accessed_time.is_some()
        || walk_data.filter_modified_time.is_some()
        || walk_data.filter_changed_time.is_some()
    {
        let size_inode_device = get_metadata(entry.path(), false, follow_links);
        if let Some((_, _, (modified_time, accessed_time, changed_time))) = size_inode_device {
            if entry.path().is_file()
                && [
                    (&walk_data.filter_modified_time, modified_time),
                    (&walk_data.filter_accessed_time, accessed_time),
                    (&walk_data.filter_changed_time, changed_time),
                ]
                .iter()
                .any(|(filter_time, actual_time)| {
                    is_filtered_out_due_to_file_time(filter_time, *actual_time)
                })
            {
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

fn walk(dir: PathBuf, walk_data: &WalkData, depth: usize) -> Option<Node> {
    let prog_data = &walk_data.progress_data;
    let errors = &walk_data.errors;

    if errors.lock().unwrap().abort {
        return None;
    }

    let children = if dir.is_dir() {
        let read_dir = fs::read_dir(&dir);
        match read_dir {
            Ok(entries) => {
                entries
                    .into_iter()
                    .par_bridge()
                    .filter_map(|entry| {
                        match entry {
                            Ok(ref entry) => {
                                // uncommenting the below line gives simpler code but
                                // rayon doesn't parallelize as well giving a 3X performance drop
                                // hence we unravel the recursion a bit

                                // return walk(entry.path(), walk_data, depth)

                                if !ignore_file(entry, walk_data) {
                                    if let Ok(data) = entry.file_type() {
                                        if data.is_dir()
                                            || (walk_data.follow_links && data.is_symlink())
                                        {
                                            return walk(entry.path(), walk_data, depth + 1);
                                        }

                                        let node = build_node(
                                            entry.path(),
                                            vec![],
                                            data.is_symlink(),
                                            data.is_file(),
                                            depth,
                                            walk_data,
                                        );

                                        prog_data.num_files.fetch_add(1, ORDERING);
                                        if let Some(ref file) = node {
                                            prog_data
                                                .total_file_size
                                                .fetch_add(file.size, ORDERING);
                                        }

                                        return node;
                                    }
                                }
                            }
                            Err(ref failed) => {
                                let mut editable_error = errors.lock().unwrap();
                                editable_error.no_permissions.insert(failed.to_string());
                            }
                        }
                        None
                    })
                    .collect()
            }
            Err(failed) => {
                let mut editable_error = errors.lock().unwrap();
                match failed.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        editable_error
                            .no_permissions
                            .insert(dir.to_string_lossy().into());
                    }
                    std::io::ErrorKind::NotFound => {
                        editable_error.file_not_found.insert(failed.to_string());
                    }
                    _ => {
                        editable_error.unknown_error.insert(failed.to_string());
                    }
                }
                vec![]
            }
        }
    } else {
        if !dir.is_file() {
            let mut editable_error = errors.lock().unwrap();
            let bad_file = dir.as_os_str().to_string_lossy().into();
            editable_error.file_not_found.insert(bad_file);
        }
        vec![]
    };
    let is_symlink = if walk_data.follow_links {
        match fs::symlink_metadata(&dir) {
            Ok(metadata) => metadata.file_type().is_symlink(),
            Err(_) => false,
        }
    } else {
        false
    };
    build_node(dir, children, is_symlink, false, depth, walk_data)
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

    #[cfg(test)]
    fn create_walker<'a>(use_apparent_size: bool) -> WalkData<'a> {
        use crate::PIndicator;
        let indicator = PIndicator::build_me();
        WalkData {
            ignore_directories: HashSet::new(),
            filter_regex: &[],
            invert_filter_regex: &[],
            allowed_filesystems: HashSet::new(),
            filter_modified_time: Some((Operater::GreaterThan, 0)),
            filter_accessed_time: Some((Operater::GreaterThan, 0)),
            filter_changed_time: Some((Operater::GreaterThan, 0)),
            use_apparent_size,
            by_filecount: false,
            by_filetime: &None,
            ignore_hidden: false,
            follow_links: false,
            progress_data: indicator.data.clone(),
            errors: Arc::new(Mutex::new(RuntimeErrors::default())),
        }
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_should_ignore_file() {
        let mut inodes = HashSet::new();
        let n = create_node();
        let walkdata = create_walker(false);

        // First time we insert the node
        assert_eq!(
            clean_inodes(n.clone(), &mut inodes, &walkdata),
            Some(n.clone())
        );

        // Second time is a duplicate - we ignore it
        assert_eq!(clean_inodes(n.clone(), &mut inodes, &walkdata), None);
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_should_not_ignore_files_if_using_apparent_size() {
        let mut inodes = HashSet::new();
        let n = create_node();
        let walkdata = create_walker(true);

        // If using apparent size we include Nodes, even if duplicate inodes
        assert_eq!(
            clean_inodes(n.clone(), &mut inodes, &walkdata),
            Some(n.clone())
        );
        assert_eq!(
            clean_inodes(n.clone(), &mut inodes, &walkdata),
            Some(n.clone())
        );
    }

    #[test]
    fn test_total_ordering_of_sort_by_inode() {
        use std::str::FromStr;

        let a = Node {
            name: PathBuf::from_str("a").unwrap(),
            size: 0,
            children: vec![],
            inode_device: Some((3, 66310)),
            depth: 0,
        };

        let b = Node {
            name: PathBuf::from_str("b").unwrap(),
            size: 0,
            children: vec![],
            inode_device: None,
            depth: 0,
        };

        let c = Node {
            name: PathBuf::from_str("c").unwrap(),
            size: 0,
            children: vec![],
            inode_device: Some((1, 66310)),
            depth: 0,
        };

        assert_eq!(sort_by_inode(&a, &b), Ordering::Greater);
        assert_eq!(sort_by_inode(&a, &c), Ordering::Greater);
        assert_eq!(sort_by_inode(&c, &b), Ordering::Greater);

        assert_eq!(sort_by_inode(&b, &a), Ordering::Less);
        assert_eq!(sort_by_inode(&c, &a), Ordering::Less);
        assert_eq!(sort_by_inode(&b, &c), Ordering::Less);
    }
}
