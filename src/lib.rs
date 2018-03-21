use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

#[derive(Clone, Debug)]
pub struct Node {
    entry: DirEnt,
    children: Vec<Node>,
}

#[derive(Clone, Debug)]
pub struct DirEnt {
    name: String,
    size: u64,
}

impl Node {
    pub fn new(entry: DirEnt, children: Vec<Node>) -> Self {
        Node {
            entry: entry,
            children: children,
        }
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn entry(&self) -> &DirEnt {
        &self.entry
    }
}

impl DirEnt {
    pub fn new(name: &str, size: u64) -> Self {
        DirEnt {
            name: String::from(name),
            size: size,
        }
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
        if self.entry.size > other.entry.size {
            Ordering::Less
        } else if self.entry.size < other.entry.size {
            Ordering::Greater
        } else {
            let my_slashes = self.entry.name.matches('/').count();
            let other_slashes = other.entry.name.matches('/').count();

            if my_slashes > other_slashes {
                Ordering::Greater
            } else if my_slashes < other_slashes {
                Ordering::Less
            } else {
                if self.entry.name < other.entry.name {
                    Ordering::Less
                } else if self.entry.name > other.entry.name {
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
        (&self.entry.name, self.entry.size) == (&other.entry.name, other.entry.size)
    }
}
impl Eq for Node {}
