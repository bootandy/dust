use crate::display_node::DisplayNode;
use crate::node::Node;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

pub fn get_biggest(
    top_level_nodes: Vec<Node>,
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
    let mut allowed_nodes = HashSet::new();

    allowed_nodes.insert(root.name.as_path());
    heap = add_children(using_a_filter, &root, depth, heap);

    for _ in number_top_level_nodes..n {
        let line = heap.pop();
        match line {
            Some(line) => {
                allowed_nodes.insert(line.name.as_path());
                heap = add_children(using_a_filter, line, depth, heap);
            }
            None => break,
        }
    }
    recursive_rebuilder(&allowed_nodes, &root)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ExtensionNode<'a> {
    size: u64,
    extension: Option<&'a OsStr>,
}

pub fn get_all_file_types(top_level_nodes: &[Node], n: usize) -> Option<DisplayNode> {
    let ext_nodes = {
        let mut extension_cumulative_sizes = HashMap::new();
        build_by_all_file_types(top_level_nodes, &mut extension_cumulative_sizes);

        let mut extension_cumulative_sizes: Vec<ExtensionNode<'_>> = extension_cumulative_sizes
            .iter()
            .map(|(&extension, &size)| ExtensionNode { extension, size })
            .collect();

        extension_cumulative_sizes.sort_by(|lhs, rhs| lhs.cmp(rhs).reverse());

        extension_cumulative_sizes
    };

    let mut ext_nodes_iter = ext_nodes.iter();

    // First, collect the first N - 1 nodes...
    let mut displayed: Vec<DisplayNode> = ext_nodes_iter
        .by_ref()
        .take(if n > 1 { n - 1 } else { 1 })
        .map(|node| DisplayNode {
            name: PathBuf::from(
                node.extension
                    .map(|ext| format!(".{}", ext.to_string_lossy()))
                    .unwrap_or_else(|| "(no extension)".to_owned()),
            ),
            size: node.size,
            children: vec![],
        })
        .collect();

    // ...then, aggregate the remaining nodes (if any) into a single  "(others)" node
    if ext_nodes_iter.len() > 0 {
        displayed.push(DisplayNode {
            name: PathBuf::from("(others)"),
            size: ext_nodes_iter.map(|node| node.size).sum(),
            children: vec![],
        });
    }

    let result = DisplayNode {
        name: PathBuf::from("(total)"),
        size: displayed.iter().map(|node| node.size).sum(),
        children: displayed,
    };

    Some(result)
}

fn add_children<'a>(
    using_a_filter: bool,
    file_or_folder: &'a Node,
    depth: usize,
    mut heap: BinaryHeap<&'a Node>,
) -> BinaryHeap<&'a Node> {
    if depth > file_or_folder.depth {
        heap.extend(
            file_or_folder
                .children
                .iter()
                .filter(|c| !using_a_filter || c.name.is_file() || c.size > 0),
        )
    }
    heap
}

fn build_by_all_file_types<'a>(
    top_level_nodes: &'a [Node],
    counter: &mut HashMap<Option<&'a OsStr>, u64>,
) {
    for node in top_level_nodes {
        if node.name.is_file() {
            let ext = node.name.extension();
            let cumulative_size = counter.entry(ext).or_default();
            *cumulative_size += node.size;
        }
        build_by_all_file_types(&node.children, counter)
    }
}

fn get_new_root(top_level_nodes: Vec<Node>) -> Node {
    if top_level_nodes.len() > 1 {
        Node {
            name: PathBuf::from("(total)"),
            size: top_level_nodes.iter().map(|node| node.size).sum(),
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
