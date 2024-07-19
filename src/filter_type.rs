use crate::display_node::DisplayNode;
use crate::node::FileTime;
use crate::node::Node;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct ExtensionNode<'a> {
    size: u64,
    extension: Option<&'a OsStr>,
}

pub fn get_all_file_types(
    top_level_nodes: &[Node],
    n: usize,
    by_filetime: &Option<FileTime>,
) -> Option<DisplayNode> {
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
        let actual_size = if by_filetime.is_some() {
            ext_nodes_iter.map(|node| node.size).max().unwrap_or(0)
        } else {
            ext_nodes_iter.map(|node| node.size).sum()
        };
        displayed.push(DisplayNode {
            name: PathBuf::from("(others)"),
            size: actual_size,
            children: vec![],
        });
    }

    let actual_size: u64 = if by_filetime.is_some() {
        displayed.iter().map(|node| node.size).max().unwrap_or(0)
    } else {
        displayed.iter().map(|node| node.size).sum()
    };

    let result = DisplayNode {
        name: PathBuf::from("(total)"),
        size: actual_size,
        children: displayed,
    };

    Some(result)
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
