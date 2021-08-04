use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Eq, Clone)]
pub struct DisplayNode {
    pub name: PathBuf, //todo: consider moving to a string?
    pub size: u64,
    pub children: Vec<DisplayNode>,
}

impl Ord for DisplayNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size == other.size {
            self.name.cmp(&other.name)
        } else {
            self.size.cmp(&other.size)
        }
    }
}

impl PartialOrd for DisplayNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DisplayNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.size == other.size && self.children == other.children
    }
}

impl DisplayNode {
    pub fn num_siblings(&self) -> u64 {
        self.children.len() as u64
    }

    pub fn get_children_from_node(&self, is_reversed: bool) -> impl Iterator<Item = DisplayNode> {
        // we box to avoid the clippy lint warning
        let out: Box<dyn Iterator<Item = DisplayNode>> = if is_reversed {
            Box::new(self.children.clone().into_iter().rev())
        } else {
            Box::new(self.children.clone().into_iter())
        };
        out
    }
}
