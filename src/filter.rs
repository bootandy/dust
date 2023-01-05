use crate::display_node::DisplayNode;
use crate::node::Node;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

pub struct AggregateData {
    pub min_size: Option<usize>,
    pub only_dir: bool,
    pub number_of_lines: usize,
    pub depth: usize,
    pub using_a_filter: bool,
}

pub fn get_biggest(top_level_nodes: Vec<Node>, display_data: AggregateData) -> Option<DisplayNode> {
    if top_level_nodes.is_empty() {
        // perhaps change this, bring back Error object?
        return None;
    }
    let mut heap = BinaryHeap::new();
    let number_top_level_nodes = top_level_nodes.len();
    let root;

    if number_top_level_nodes > 1 {
        let size = top_level_nodes.iter().map(|node| node.size).sum();
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

    let nol = display_data.number_of_lines;
    let remaining = nol.saturating_sub(number_top_level_nodes);
    fill_remaining_lines(heap, &root, remaining, display_data)
}

pub fn fill_remaining_lines<'a>(
    mut heap: BinaryHeap<&'a Node>,
    root: &'a Node,
    remaining_lines: usize,
    display_data: AggregateData,
) -> Option<DisplayNode> {
    let mut allowed_nodes = HashSet::new();
    allowed_nodes.insert(root.name.as_path());

    for _ in 0..remaining_lines {
        let line = heap.pop();
        match line {
            Some(line) => {
                allowed_nodes.insert(line.name.as_path());
                heap = add_children(&display_data, line, heap);
            }
            None => break,
        }
    }
    recursive_rebuilder(&allowed_nodes, root)
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
