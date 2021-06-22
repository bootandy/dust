use crate::display_node::DisplayNode;
use crate::node::Node;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn get_by_depth(top_level_nodes: Vec<Node>, n: usize) -> Option<DisplayNode> {
    if top_level_nodes.is_empty() {
        // perhaps change this, bring back Error object?
        return None;
    }
    let root = get_new_root(top_level_nodes);
    Some(build_by_depth(&root, n - 1))
}

pub fn get_biggest(top_level_nodes: Vec<Node>, n: usize) -> Option<DisplayNode> {
    if top_level_nodes.is_empty() {
        // perhaps change this, bring back Error object?
        return None;
    }

    let mut heap = BinaryHeap::new();
    let number_top_level_nodes = top_level_nodes.len();
    let root = get_new_root(top_level_nodes);

    root.children.iter().for_each(|c| heap.push(c));

    let mut allowed_nodes = HashSet::new();
    allowed_nodes.insert(&root.name);

    for _ in number_top_level_nodes..n {
        let line = heap.pop();
        match line {
            Some(line) => {
                line.children.iter().for_each(|c| heap.push(c));
                allowed_nodes.insert(&line.name);
            }
            None => break,
        }
    }
    recursive_rebuilder(&allowed_nodes, &root)
}

fn build_by_depth(node: &Node, depth: usize) -> DisplayNode {
    let new_children = {
        if depth == 0 {
            vec![]
        } else {
            let mut new_children: Vec<_> = node
                .children
                .iter()
                .map(|c| build_by_depth(c, depth - 1))
                .collect();
            new_children.sort();
            new_children.reverse();
            new_children
        }
    };

    DisplayNode {
        name: node.name.clone(),
        size: node.size,
        children: new_children,
    }
}

fn get_new_root(top_level_nodes: Vec<Node>) -> Node {
    if top_level_nodes.len() > 1 {
        let total_size = top_level_nodes.iter().map(|node| node.size).sum();
        Node {
            name: PathBuf::from("(total)"),
            size: total_size,
            children: top_level_nodes,
            inode_device: None,
        }
    } else {
        top_level_nodes.into_iter().next().unwrap()
    }
}

fn recursive_rebuilder<'a>(
    allowed_nodes: &'a HashSet<&PathBuf>,
    current: &Node,
) -> Option<DisplayNode> {
    let mut new_children: Vec<_> = current
        .children
        .iter()
        .filter_map(|c| {
            if allowed_nodes.contains(&c.name) {
                recursive_rebuilder(allowed_nodes, c)
            } else {
                None
            }
        })
        .collect();
    new_children.sort();
    new_children.reverse();
    let newnode = DisplayNode {
        name: current.name.clone(),
        size: current.size,
        children: new_children,
    };
    Some(newnode)
}
