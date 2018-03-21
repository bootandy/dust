use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

#[derive(Clone, Debug)]
pub struct Node {
    pub dir: DirEnt,
    pub children: Vec<Node>,
}

#[derive(Clone, Debug)]
pub struct DirEnt {
    pub name: String,
    pub size: u64,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.dir.size > other.dir.size {
            Ordering::Less
        } else if self.dir.size < other.dir.size {
            Ordering::Greater
        } else {
            let my_slashes = self.dir.name.matches('/').count();
            let other_slashes = other.dir.name.matches('/').count();

            if my_slashes > other_slashes {
                Ordering::Greater
            } else if my_slashes < other_slashes {
                Ordering::Less
            } else {
                if self.dir.name < other.dir.name {
                    Ordering::Less
                } else if self.dir.name > other.dir.name {
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
        (&self.dir.name, self.dir.size) == (&other.dir.name, other.dir.size)
    }
}
impl Eq for Node {}
