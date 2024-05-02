use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize)]
pub struct DisplayNode {
    // Note: the order of fields in important here, for PartialEq and PartialOrd
    pub size: u64,
    pub name: PathBuf,
    pub children: Vec<DisplayNode>,
}

impl DisplayNode {
    pub fn num_siblings(&self) -> u64 {
        self.children.len() as u64
    }

    pub fn get_children_from_node(&self, is_reversed: bool) -> impl Iterator<Item = &DisplayNode> {
        // we box to avoid the clippy lint warning
        let out: Box<dyn Iterator<Item = &DisplayNode>> = if is_reversed {
            Box::new(self.children.iter().rev())
        } else {
            Box::new(self.children.iter())
        };
        out
    }
}
