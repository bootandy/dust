use stfu8::encode_u8;

use crate::display::get_printable_name;
use crate::display_node::DisplayNode;
use crate::node::FileTime;
use crate::node::Node;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

pub struct AggregateData {
    pub min_size: Option<usize>,
    pub only_dir: bool,
    pub only_file: bool,
    pub number_of_lines: usize,
    pub depth: usize,
    pub using_a_filter: bool,
    pub short_paths: bool,
}

pub fn get_biggest(
    top_level_nodes: Vec<Node>,
    display_data: AggregateData,
    by_filetime: &Option<FileTime>,
    keep_collapsed: HashSet<PathBuf>,
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

        let nodes = handle_duplicate_top_level_names(top_level_nodes, display_data.short_paths);

        root = Node {
            name: PathBuf::from("(total)"),
            size,
            children: nodes,
            inode_device: None,
            depth: 0,
        };

        // Always include the base nodes if we add a 'parent' (total) node
        heap = always_add_children(&display_data, &root, heap);
    } else {
        root = top_level_nodes.into_iter().next().unwrap();
        heap = add_children(&display_data, &root, heap);
    }

    Some(fill_remaining_lines(
        heap,
        &root,
        display_data,
        keep_collapsed,
    ))
}

pub fn fill_remaining_lines<'a>(
    mut heap: BinaryHeap<&'a Node>,
    root: &'a Node,
    display_data: AggregateData,
    keep_collapsed: HashSet<PathBuf>,
) -> DisplayNode {
    let mut allowed_nodes = HashMap::new();

    while allowed_nodes.len() < display_data.number_of_lines {
        let line = heap.pop();
        match line {
            Some(line) => {
                // If we are not doing only_file OR if we are doing
                // only_file and it has no children (ie is a file not a dir)
                if !display_data.only_file || line.children.is_empty() {
                    allowed_nodes.insert(line.name.as_path(), line);
                }
                if !keep_collapsed.contains(&line.name) {
                    heap = add_children(&display_data, line, heap);
                }
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

fn names_have_dup(top_level_nodes: &Vec<Node>) -> bool {
    let mut stored = HashSet::new();
    for node in top_level_nodes {
        let name = get_printable_name(&node.name, true);
        if stored.contains(&name) {
            return true;
        }
        stored.insert(name);
    }
    false
}

fn handle_duplicate_top_level_names(top_level_nodes: Vec<Node>, short_paths: bool) -> Vec<Node> {
    // If we have top level names that are the same - we need to tweak them:
    if short_paths && names_have_dup(&top_level_nodes) {
        let mut new_top_nodes = top_level_nodes.clone();
        let mut dir_walk_up_count = 0;

        while names_have_dup(&new_top_nodes) && dir_walk_up_count < 10 {
            dir_walk_up_count += 1;
            let mut newer = vec![];

            for node in new_top_nodes.iter() {
                let mut folders = node.name.iter().rev();
                // Get parent folder (if second time round get grandparent and so on)
                for _ in 0..dir_walk_up_count {
                    folders.next();
                }
                match folders.next() {
                    // Add (parent_name) to path of Node
                    Some(data) => {
                        let parent = encode_u8(data.as_encoded_bytes());
                        let current_node = node.name.display();
                        let n = Node {
                            name: PathBuf::from(format!("{current_node}({parent})")),
                            size: node.size,
                            children: node.children.clone(),
                            inode_device: node.inode_device,
                            depth: node.depth,
                        };
                        newer.push(n)
                    }
                    // Node does not have a parent
                    None => newer.push(node.clone()),
                }
            }
            new_top_nodes = newer;
        }
        new_top_nodes
    } else {
        top_level_nodes
    }
}
