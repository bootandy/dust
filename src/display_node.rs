use std::cell::RefCell;
use std::path::PathBuf;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::display::human_readable_number;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

// Only used for -j 'json' flag combined with -o 'output_type' flag
// Used to pass the output_type into the custom Serde serializer
thread_local! {
    pub static OUTPUT_TYPE: RefCell<String> = const { RefCell::new(String::new()) };
}

/*
We need the custom Serialize incase someone uses the -o flag to pass a custom output type in
(show size in Mb / Gb etc).
Sadly this also necessitates a global variable OUTPUT_TYPE as we can not pass the output_type flag
into the serialize method
 */
impl Serialize for DisplayNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let readable_size = OUTPUT_TYPE
            .with(|output_type| human_readable_number(self.size, output_type.borrow().as_str()));
        let mut state = serializer.serialize_struct("DisplayNode", 2)?;
        state.serialize_field("size", &(readable_size))?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}
