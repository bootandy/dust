use crate::platform::get_metadata;
use crate::utils::is_filtered_out_due_to_extension;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Eq, Clone)]
pub struct Node {
    pub name: PathBuf,
    pub size: u64,
    pub children: Vec<Node>,
    pub inode_device: Option<(u64, u64)>,
}

pub fn build_node(
    dir: PathBuf,
    children: Vec<Node>,
    filtered_extensions: &HashSet<&str>,
    use_apparent_size: bool,
    is_symlink: bool,
    by_filecount: bool,
) -> Option<Node> {
    match get_metadata(&dir, use_apparent_size) {
        Some(data) => {
            let inode_device = if is_symlink && !use_apparent_size {
                None
            } else {
                data.1
            };

            let size = if is_filtered_out_due_to_extension(filtered_extensions, &dir)
                || (is_symlink && !use_apparent_size)
            {
                0
            } else if by_filecount {
                1
            } else {
                data.0
            };

            Some(Node {
                name: dir,
                size,
                children,
                inode_device,
            })
        }
        None => None,
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.children == other.children
    }
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
