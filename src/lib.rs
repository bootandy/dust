use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

#[derive(Clone, Debug)]
pub struct Node {
    name: String,
    size: u64,
    children: Vec<Node>,
}

impl Node {
    pub fn new<S: Into<String>>(name: S, size: u64, children: Vec<Node>) -> Self {
        Node {
            children: children,
            name: name.into(),
            size: size,
        }
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size > other.size {
            Ordering::Less
        } else if self.size < other.size {
            Ordering::Greater
        } else {
            let my_slashes = self.name.matches('/').count();
            let other_slashes = other.name.matches('/').count();

            if my_slashes > other_slashes {
                Ordering::Greater
            } else if my_slashes < other_slashes {
                Ordering::Less
            } else {
                if self.name < other.name {
                    Ordering::Less
                } else if self.name > other.name {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
        }
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        (&self.name, self.size) == (&other.name, other.size)
    }
}
impl Eq for Node {}
