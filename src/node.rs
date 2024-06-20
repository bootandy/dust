use crate::dir_walker::WalkData;
use crate::platform::get_metadata;
use crate::utils::is_filtered_out_due_to_file_time;
use crate::utils::is_filtered_out_due_to_invert_regex;
use crate::utils::is_filtered_out_due_to_regex;

use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Eq, Clone)]
pub struct Node {
    pub name: PathBuf,
    pub size: u64,
    pub children: Vec<Node>,
    pub inode_device: Option<(u64, u64)>,
    pub depth: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn build_node(
    dir: PathBuf,
    children: Vec<Node>,
    is_symlink: bool,
    is_file: bool,
    depth: usize,
    walk_data: &WalkData,
) -> Option<Node> {
    let use_apparent_size = walk_data.use_apparent_size;
    let by_filecount = walk_data.by_filecount;

    get_metadata(&dir, use_apparent_size).map(|data| {
        let inode_device = if is_symlink && !use_apparent_size {
            None
        } else {
            data.1
        };

        let size = if is_filtered_out_due_to_regex(walk_data.filter_regex, &dir)
            || is_filtered_out_due_to_invert_regex(walk_data.invert_filter_regex, &dir)
            || (is_symlink && !use_apparent_size)
            || by_filecount && !is_file
            || [
                (&walk_data.filter_modified_time, data.2 .0),
                (&walk_data.filter_accessed_time, data.2 .1),
                (&walk_data.filter_changed_time, data.2 .2),
            ]
            .iter()
            .any(|(filter_time, actual_time)| {
                is_filtered_out_due_to_file_time(filter_time, *actual_time)
            }) {
            0
        } else if by_filecount {
            1
        } else {
            data.0
        };

        Node {
            name: dir,
            size,
            children,
            inode_device,
            depth,
        }
    })
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.children == other.children
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size
            .cmp(&other.size)
            .then_with(|| self.name.cmp(&other.name))
            .then_with(|| self.children.cmp(&other.children))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
