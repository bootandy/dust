use crate::display_node::DisplayNode;
use crate::node::Node;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

pub fn get_biggest(
    top_level_nodes: Vec<Node>,
    min_size: Option<usize>,
    only_dir: bool,
    n: usize,
    depth: usize,
    using_a_filter: bool,
) -> Option<DisplayNode> {
    if top_level_nodes.is_empty() {
        // perhaps change this, bring back Error object?
        return None;
    }
    let mut heap = BinaryHeap::new();
    let number_top_level_nodes = top_level_nodes.len();

    let root = get_new_root(top_level_nodes);

    if number_top_level_nodes > 1 {
        heap = add_children(using_a_filter, min_size, only_dir, &root, usize::MAX, heap);
    } else {
        heap = add_children(using_a_filter, min_size, only_dir, &root, depth, heap);
    }
    let remaining = n.checked_sub(number_top_level_nodes).unwrap_or(0);
    fill_remaining_lines(
        heap,
        &root,
        min_size,
        only_dir,
        remaining,
        depth,
        using_a_filter,
    )
}

pub fn fill_remaining_lines<'a>(
    mut heap: BinaryHeap<&'a Node>,
    root: &'a Node,
    min_size: Option<usize>,
    only_dir: bool,
    remaining: usize,
    depth: usize,
    using_a_filter: bool,
) -> Option<DisplayNode> {
    let mut allowed_nodes = HashSet::new();
    allowed_nodes.insert(root.name.as_path());

    for _ in 0..remaining {
        let line = heap.pop();
        match line {
            Some(line) => {
                allowed_nodes.insert(line.name.as_path());
                heap = add_children(using_a_filter, min_size, only_dir, line, depth, heap);
            }
            None => break,
        }
    }
    recursive_rebuilder(&allowed_nodes, &root)
}

fn add_children<'a>(
    using_a_filter: bool,
    min_size: Option<usize>,
    only_dir: bool,
    file_or_folder: &'a Node,
    depth: usize,
    mut heap: BinaryHeap<&'a Node>,
) -> BinaryHeap<&'a Node> {
    if depth > file_or_folder.depth {
        heap.extend(
            file_or_folder
                .children
                .iter()
                .filter(|c| match min_size {
                    Some(ms) => c.size > ms as u64,
                    None => !using_a_filter || c.name.is_file() || c.size > 0,
                })
                .filter(|c| if only_dir { c.name.is_dir() } else { true }),
        )
    }
    heap
}

fn get_new_root(top_level_nodes: Vec<Node>) -> Node {
    if top_level_nodes.len() != 1 {
        let size = top_level_nodes.iter().map(|node| node.size).sum();
        Node {
            name: PathBuf::from("(total)"),
            size,
            children: top_level_nodes,
            inode_device: None,
            depth: 0,
        }
    } else {
        top_level_nodes.into_iter().next().unwrap()
    }
}

fn recursive_rebuilder(allowed_nodes: &HashSet<&Path>, current: &Node) -> Option<DisplayNode> {
    let mut new_children: Vec<_> = current
        .children
        .iter()
        .filter(|c| allowed_nodes.contains(c.name.as_path()))
        .filter_map(|c| recursive_rebuilder(allowed_nodes, c))
        .collect();

    new_children.sort_by(|lhs, rhs| lhs.cmp(rhs).reverse());

    Some(DisplayNode {
        name: current.name.clone(),
        size: current.size,
        children: new_children,
    })
}
