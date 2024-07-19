use crate::display_node::DisplayNode;
use crate::node::FileTime;
use crate::node::Node;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

pub struct AggregateData {
    pub min_size: Option<usize>,
    pub only_dir: bool,
    pub only_file: bool,
    pub number_of_lines: usize,
    pub depth: usize,
    pub using_a_filter: bool,
}

pub fn get_biggest(
    top_level_nodes: Vec<Node>,
    display_data: AggregateData,
    by_filetime: &Option<FileTime>,
) -> Option<DisplayNode> {
    if top_level_nodes.is_empty() {
        // perhaps change this, bring back Error object?
        return None;
    }
    let mut heap = BinaryHeap::new();
    let number_top_level_nodes = top_level_nodes.len();
    let root;

    if number_top_level_nodes > 1 {
        let size = if by_filetime.is_some() {
            top_level_nodes
                .iter()
                .map(|node| node.size)
                .max()
                .unwrap_or(0)
        } else {
            top_level_nodes.iter().map(|node| node.size).sum()
        };
        root = Node {
            name: PathBuf::from("(total)"),
            size,
            children: top_level_nodes,
            inode_device: None,
            depth: 0,
        };
        // Always include the base nodes if we add a 'parent' (total) node
        heap = always_add_children(&display_data, &root, heap);
    } else {
        root = top_level_nodes.into_iter().next().unwrap();
        heap = add_children(&display_data, &root, heap);
    }

    Some(fill_remaining_lines(heap, &root, display_data))
}

pub fn fill_remaining_lines<'a>(
    mut heap: BinaryHeap<&'a Node>,
    root: &'a Node,
    display_data: AggregateData,
) -> DisplayNode {
    let mut allowed_nodes = HashMap::new();

    while allowed_nodes.len() < display_data.number_of_lines {
        let line = heap.pop();
        match line {
            Some(line) => {
                if !display_data.only_file || line.children.is_empty() {
                    allowed_nodes.insert(line.name.as_path(), line);
                }
                heap = add_children(&display_data, line, heap);
            }
            None => break,
        }
    }

    if display_data.only_file {
        flat_rebuilder(allowed_nodes, root)
    } else {
        recursive_rebuilder(&allowed_nodes, root)
    }
}

fn add_children<'a>(
    display_data: &AggregateData,
    file_or_folder: &'a Node,
    heap: BinaryHeap<&'a Node>,
) -> BinaryHeap<&'a Node> {
    if display_data.depth > file_or_folder.depth {
        always_add_children(display_data, file_or_folder, heap)
    } else {
        heap
    }
}

fn always_add_children<'a>(
    display_data: &AggregateData,
    file_or_folder: &'a Node,
    mut heap: BinaryHeap<&'a Node>,
) -> BinaryHeap<&'a Node> {
    heap.extend(
        file_or_folder
            .children
            .iter()
            .filter(|c| match display_data.min_size {
                Some(ms) => c.size > ms as u64,
                None => !display_data.using_a_filter || c.name.is_file() || c.size > 0,
            })
            .filter(|c| {
                if display_data.only_dir {
                    c.name.is_dir()
                } else {
                    true
                }
            }),
    );
    heap
}

// Finds children of current, if in allowed_nodes adds them as children to new DisplayNode
fn recursive_rebuilder(allowed_nodes: &HashMap<&Path, &Node>, current: &Node) -> DisplayNode {
    let new_children: Vec<_> = current
        .children
        .iter()
        .filter(|c| allowed_nodes.contains_key(c.name.as_path()))
        .map(|c| recursive_rebuilder(allowed_nodes, c))
        .collect();

    build_display_node(new_children, current)
}

// Applies all allowed nodes as children to current node
fn flat_rebuilder(allowed_nodes: HashMap<&Path, &Node>, current: &Node) -> DisplayNode {
    let new_children: Vec<DisplayNode> = allowed_nodes
        .into_values()
        .map(|v| DisplayNode {
            name: v.name.clone(),
            size: v.size,
            children: vec![],
        })
        .collect::<Vec<DisplayNode>>();
    build_display_node(new_children, current)
}

fn build_display_node(mut new_children: Vec<DisplayNode>, current: &Node) -> DisplayNode {
    new_children.sort_by(|lhs, rhs| lhs.cmp(rhs).reverse());
    DisplayNode {
        name: current.name.clone(),
        size: current.size,
        children: new_children,
    }
}
