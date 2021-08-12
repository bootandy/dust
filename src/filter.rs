use crate::display_node::DisplayNode;
use crate::node::Node;
use std::collections::BinaryHeap;
use std::collections::HashMap;
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

pub fn get_biggest(
    top_level_nodes: Vec<Node>,
    n: usize,
    using_file_type_filter: bool,
) -> Option<DisplayNode> {
    if top_level_nodes.is_empty() {
        // perhaps change this, bring back Error object?
        return None;
    }

    let mut heap = BinaryHeap::new();
    let number_top_level_nodes = top_level_nodes.len();
    let root = get_new_root(top_level_nodes);
    let mut allowed_nodes = HashSet::new();

    allowed_nodes.insert(&root.name);
    heap = add_children(using_file_type_filter, &root, heap);

    for _ in number_top_level_nodes..n {
        let line = heap.pop();
        match line {
            Some(line) => {
                allowed_nodes.insert(&line.name);
                heap = add_children(using_file_type_filter, line, heap);
            }
            None => break,
        }
    }
    recursive_rebuilder(&allowed_nodes, &root)
}

pub fn get_all_file_types(top_level_nodes: Vec<Node>, n: usize) -> Option<DisplayNode> {
    let mut map: HashMap<String, DisplayNode> = HashMap::new();
    build_by_all_file_types(top_level_nodes, &mut map);
    let mut by_types: Vec<DisplayNode> = map.into_iter().map(|(_k, v)| v).collect();
    by_types.sort();
    by_types.reverse();

    let displayed = if by_types.len() <= n {
        by_types
    } else {
        let (displayed, rest) = by_types.split_at(if n > 1 { n - 1 } else { 1 });
        let remaining = DisplayNode {
            name: PathBuf::from("(others)"),
            size: rest.iter().map(|a| a.size).sum(),
            children: vec![],
        };

        let mut displayed = displayed.to_vec();
        displayed.push(remaining);
        displayed
    };

    let result = DisplayNode {
        name: PathBuf::from("(total)"),
        size: displayed.iter().map(|a| a.size).sum(),
        children: displayed,
    };
    Some(result)
}

fn add_children<'a>(
    using_file_type_filter: bool,
    line: &'a Node,
    mut heap: BinaryHeap<&'a Node>,
) -> BinaryHeap<&'a Node> {
    if using_file_type_filter {
        line.children.iter().for_each(|c| {
            if !c.name.is_file() && c.size > 0 {
                heap.push(c)
            }
        });
    } else {
        line.children.iter().for_each(|c| heap.push(c));
    }
    heap
}

fn build_by_all_file_types(top_level_nodes: Vec<Node>, counter: &mut HashMap<String, DisplayNode>) {
    for node in top_level_nodes {
        if node.name.is_file() {
            let ext = node.name.extension();
            let key: String = match ext {
                Some(e) => ".".to_string() + &e.to_string_lossy(),
                None => "(no extension)".into(),
            };
            let mut display_node = counter.entry(key.clone()).or_insert(DisplayNode {
                name: PathBuf::from(key),
                size: 0,
                children: vec![],
            });
            display_node.size += node.size;
        }
        build_by_all_file_types(node.children, counter)
    }
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
