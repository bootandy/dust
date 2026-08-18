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
) -> DisplayNode {
    let ext_nodes = {
        let mut extension_cumulative_sizes = HashMap::new();
        build_by_all_file_types(
            top_level_nodes,
            &mut extension_cumulative_sizes,
            by_filetime,
        );

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
        // '(others)' is the sum of the remaining nodes so it can be bigger than
        // the nodes above it: re-sort so the tree stays in size order.
        displayed.sort_by(|lhs, rhs| lhs.cmp(rhs).reverse());
    }

    let actual_size: u64 = if by_filetime.is_some() {
        displayed.iter().map(|node| node.size).max().unwrap_or(0)
    } else {
        displayed.iter().map(|node| node.size).sum()
    };

    DisplayNode {
        name: PathBuf::from("(total)"),
        size: actual_size,
        children: displayed,
    }
}

fn build_by_all_file_types<'a>(
    top_level_nodes: &'a [Node],
    counter: &mut HashMap<Option<&'a OsStr>, u64>,
    by_filetime: &Option<FileTime>,
) {
    for node in top_level_nodes {
        if node.name.is_file() {
            let ext = node.name.extension();
            let cumulative_size = counter.entry(ext).or_default();
            if by_filetime.is_some() {
                // 'size' is a timestamp, summing them is meaningless
                *cumulative_size = (*cumulative_size).max(node.size);
            } else {
                *cumulative_size += node.size;
            }
        }
        build_by_all_file_types(&node.children, counter, by_filetime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_node(name: &str, size: u64) -> Node {
        Node {
            name: PathBuf::from(name),
            size,
            children: vec![],
            inode_device: None,
            depth: 1,
        }
    }

    #[test]
    fn test_others_node_is_sorted_by_size() {
        // These must be real files: only files are counted by extension.
        // Their sizes come from the Node, not from disk.
        let nodes = vec![
            file_node("Cargo.toml", 40),
            file_node("README.md", 30),
            file_node("build.rs", 20),
            file_node("Cargo.lock", 10),
        ];

        // 2 lines: '.toml' then '(others)', which sums to 60 and so must sort first
        let tree = get_all_file_types(&nodes, 2, &None);

        assert_eq!(tree.size, 100);
        let displayed: Vec<_> = tree
            .children
            .iter()
            .map(|c| (c.name.to_string_lossy().to_string(), c.size))
            .collect();
        assert_eq!(
            displayed,
            vec![("(others)".to_string(), 60), (".toml".to_string(), 40)]
        );
    }
}
