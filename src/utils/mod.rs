use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use ignore::{WalkBuilder, WalkState};
use std::sync::atomic;
use std::thread;

mod platform;
use self::platform::*;

type PathData = (PathBuf, u64, Option<(u64, u64)>);

#[derive(Debug, Default, Eq, Clone)]
pub struct Node {
    pub name: PathBuf,
    pub size: u64,
    pub children: Vec<Node>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size == other.size {
            self.name.cmp(&other.name)
        } else {
            self.size.cmp(&other.size)
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
        self.name == other.name && self.size == other.size && self.children == other.children
    }
}

impl Node {
    pub fn num_siblings(&self) -> u64 {
        self.children.len() as u64
    }

    pub fn get_children_from_node(&self, is_reversed: bool) -> impl Iterator<Item = Node> {
        if is_reversed {
            let children: Vec<Node> = self.children.clone().into_iter().rev().collect();
            children.into_iter()
        } else {
            self.children.clone().into_iter()
        }
    }
}

pub struct Errors {
    pub permissions: bool,
    pub not_found: bool,
}

pub fn is_a_parent_of<P: AsRef<Path>>(parent: P, child: P) -> bool {
    let parent = parent.as_ref();
    let child = child.as_ref();
    child.starts_with(parent) && !parent.starts_with(child)
}

pub fn simplify_dir_names<P: AsRef<Path>>(filenames: Vec<P>) -> HashSet<PathBuf> {
    let mut top_level_names: HashSet<PathBuf> = HashSet::with_capacity(filenames.len());
    let mut to_remove: Vec<PathBuf> = Vec::with_capacity(filenames.len());

    for t in filenames {
        let top_level_name = normalize_path(t);
        let mut can_add = true;

        for tt in top_level_names.iter() {
            if is_a_parent_of(&top_level_name, tt) {
                to_remove.push(tt.to_path_buf());
            } else if is_a_parent_of(tt, &top_level_name) {
                can_add = false;
            }
        }
        to_remove.sort_unstable();
        top_level_names.retain(|tr| to_remove.binary_search(tr).is_err());
        to_remove.clear();
        if can_add {
            top_level_names.insert(top_level_name);
        }
    }

    top_level_names
}

fn prepare_walk_dir_builder<P: AsRef<Path>>(
    top_level_names: &HashSet<P>,
    limit_filesystem: bool,
    show_hidden: bool,
) -> WalkBuilder {
    let mut it = top_level_names.iter();
    let mut builder = WalkBuilder::new(it.next().unwrap());
    builder.follow_links(false);
    if show_hidden {
        builder.hidden(false);
        builder.ignore(false);
        builder.git_global(false);
        builder.git_ignore(false);
        builder.git_exclude(false);
    }

    if limit_filesystem {
        builder.same_file_system(true);
    }

    for b in it {
        builder.add(b);
    }
    builder
}

fn is_not_found(e: &ignore::Error) -> bool {
    use ignore::Error;
    if let Error::WithPath { err, .. } = e {
        if let Error::Io(e) = &**err {
            if e.kind() == std::io::ErrorKind::NotFound {
                return true;
            }
        }
    }
    false
}

pub fn get_dir_tree<P: AsRef<Path>>(
    top_level_names: &HashSet<P>,
    ignore_directories: &Option<Vec<PathBuf>>,
    apparent_size: bool,
    limit_filesystem: bool,
    by_filecount: bool,
    show_hidden: bool,
) -> (Errors, HashMap<PathBuf, u64>) {
    let permissions_flag = AtomicBool::new(false);
    let not_found_flag = AtomicBool::new(false);

    let t2 = top_level_names
        .iter()
        .map(|p| p.as_ref().to_path_buf())
        .collect();

    let (gather_entry_owned, aggregate_entries) = create_reader(t2, apparent_size);
    let gather_entry = &gather_entry_owned;
    let walk_dir_builder = prepare_walk_dir_builder(top_level_names, limit_filesystem, show_hidden);
    let pf = &permissions_flag;
    let nf = &not_found_flag;
    let gather_entry = &gather_entry;
    let process_entry = Box::new(move |path| {
        match path {
            Ok(p) => {
                let p: ignore::DirEntry = p;
                if let Some(dirs) = ignore_directories {
                    let path = p.path();
                    let parts = path.components().collect::<Vec<std::path::Component>>();
                    for d in dirs {
                        if parts
                            .windows(d.components().count())
                            .any(|window| window.iter().collect::<PathBuf>() == *d)
                        {
                            return WalkState::Continue;
                        }
                    }
                }

                let maybe_size_and_inode = get_metadata(&p, apparent_size);

                match maybe_size_and_inode {
                    Some(data) => {
                        let (size, inode_device) = if by_filecount { (1, data.1) } else { data };
                        gather_entry((p.into_path(), size, inode_device));
                    }
                    None => {
                        pf.store(true, atomic::Ordering::Relaxed);
                    }
                }
            }
            Err(e) => {
                if is_not_found(&e) {
                    nf.store(true, atomic::Ordering::Relaxed);
                } else {
                    pf.store(true, atomic::Ordering::Relaxed);
                }
            }
        };
        WalkState::Continue
    });

    #[cfg(not(target_arch = "wasm32"))]
    walk_dir_builder
        .build_parallel()
        .run(move || process_entry.clone());
    #[cfg(target_arch = "wasm32")]
    walk_dir_builder.build().for_each(|e| {
        process_entry(e);
    });
    std::mem::drop(gather_entry_owned);

    let data = aggregate_entries();
    let errors = Errors {
        permissions: permissions_flag.load(atomic::Ordering::SeqCst),
        not_found: not_found_flag.load(atomic::Ordering::SeqCst),
    };
    (errors, data)
}

type ReaderAggregators = (
    Box<dyn Fn(PathData) + Sync>,
    Box<dyn FnOnce() -> HashMap<PathBuf, u64>>,
);
#[cfg(not(target_arch = "wasm32"))]
fn create_reader(top_level_names: HashSet<PathBuf>, apparent_size: bool) -> ReaderAggregators {
    let (tx, rx) = channel::bounded::<PathData>(1000);

    // Receiver thread
    let hnd = thread::spawn(move || {
        let mut hash: HashMap<PathBuf, u64> = HashMap::new();
        let mut inodes: HashSet<(u64, u64)> = HashSet::new();

        for dent in rx {
            read_info(
                dent,
                &mut hash,
                &mut inodes,
                &top_level_names,
                apparent_size,
            );
        }
        hash
    });

    (
        Box::new(move |e| tx.send(e).unwrap()),
        Box::new(move || hnd.join().unwrap()),
    )
}

#[cfg(target_arch = "wasm32")]
fn create_reader(
    top_level_names: HashSet<PathBuf>,
    apparent_size: bool,
) -> (
    Box<dyn Fn(PathData)>,
    Box<dyn FnOnce() -> HashMap<PathBuf, u64>>,
) {
    use std::cell::RefCell;
    use std::rc::Rc;
    let hash: Rc<RefCell<HashMap<PathBuf, u64>>> = Rc::new(RefCell::new(HashMap::new()));
    let inodes: Rc<RefCell<HashSet<(u64, u64)>>> = Rc::new(RefCell::new(HashSet::new()));
    let hash_ret = hash.clone();

    (
        Box::new(move |info| {
            read_info(
                info,
                &mut hash.borrow_mut(),
                &mut inodes.borrow_mut(),
                &top_level_names,
                apparent_size,
            )
        }),
        Box::new(move || Rc::try_unwrap(hash_ret).unwrap().into_inner()),
    )
}

fn read_info(
    (path, size, maybe_inode_device): (PathBuf, u64, Option<(u64, u64)>),
    hash: &mut HashMap<PathBuf, u64>,
    inodes: &mut HashSet<(u64, u64)>,
    top_level_names: &HashSet<PathBuf>,
    apparent_size: bool,
) {
    if !should_ignore_file(apparent_size, inodes, maybe_inode_device) {
        for p in path.ancestors() {
            let s = hash.entry(p.to_path_buf()).or_insert(0);
            *s += size;

            if top_level_names.contains(p) {
                break;
            }
        }
    }
}

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    // normalize path ...
    // 1. removing repeated separators
    // 2. removing interior '.' ("current directory") path segments
    // 3. removing trailing extra separators and '.' ("current directory") path segments
    // * `Path.components()` does all the above work; ref: <https://doc.rust-lang.org/std/path/struct.Path.html#method.components>
    // 4. changing to os preferred separator (automatically done by recollecting components back into a PathBuf)
    path.as_ref().components().collect::<PathBuf>()
}

fn should_ignore_file(
    apparent_size: bool,
    inodes: &mut HashSet<(u64, u64)>,
    maybe_inode_device: Option<(u64, u64)>,
) -> bool {
    match maybe_inode_device {
        None => false,
        Some(data) => {
            let (inode, device) = data;
            if !apparent_size {
                // Ignore files already visited or symlinked
                if inodes.contains(&(inode, device)) {
                    return true;
                }
                inodes.insert((inode, device));
            }
            false
        }
    }
}

pub fn sort_by_size_first_name_second(a: &(PathBuf, u64), b: &(PathBuf, u64)) -> Ordering {
    let result = b.1.cmp(&a.1);
    if result == Ordering::Equal {
        a.0.cmp(&b.0)
    } else {
        result
    }
}

pub fn sort(data: HashMap<PathBuf, u64>) -> Vec<(PathBuf, u64)> {
    let mut new_l: Vec<(PathBuf, u64)> = data.iter().map(|(a, b)| (a.clone(), *b)).collect();
    new_l.sort_unstable_by(sort_by_size_first_name_second);
    new_l
}

pub fn find_big_ones(new_l: Vec<(PathBuf, u64)>, max_to_show: usize) -> Vec<(PathBuf, u64)> {
    if max_to_show > 0 && new_l.len() > max_to_show {
        new_l[0..max_to_show].to_vec()
    } else {
        new_l
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_simplify_dir() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("a"));
        assert_eq!(simplify_dir_names(vec!["a"]), correct);
    }

    #[test]
    fn test_simplify_dir_rm_subdir() {
        let mut correct = HashSet::new();
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        assert_eq!(simplify_dir_names(vec!["a/b", "a/b/c", "a/b/d/f"]), correct);
    }

    #[test]
    fn test_simplify_dir_duplicates() {
        let mut correct = HashSet::new();
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        correct.insert(PathBuf::from("c"));
        assert_eq!(
            simplify_dir_names(vec![
                "a/b",
                "a/b//",
                "a/././b///",
                "c",
                "c/",
                "c/.",
                "c/././",
                "c/././."
            ]),
            correct
        );
    }
    #[test]
    fn test_simplify_dir_rm_subdir_and_not_substrings() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("b"));
        correct.insert(["c", "a", "b"].iter().collect::<PathBuf>());
        correct.insert(["a", "b"].iter().collect::<PathBuf>());
        assert_eq!(simplify_dir_names(vec!["a/b", "c/a/b/", "b"]), correct);
    }

    #[test]
    fn test_simplify_dir_dots() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("src"));
        assert_eq!(simplify_dir_names(vec!["src/."]), correct);
    }

    #[test]
    fn test_simplify_dir_substring_names() {
        let mut correct = HashSet::new();
        correct.insert(PathBuf::from("src"));
        correct.insert(PathBuf::from("src_v2"));
        assert_eq!(simplify_dir_names(vec!["src/", "src_v2"]), correct);
    }

    #[test]
    fn test_is_a_parent_of() {
        assert!(is_a_parent_of("/usr", "/usr/andy"));
        assert!(is_a_parent_of("/usr", "/usr/andy/i/am/descendant"));
        assert!(!is_a_parent_of("/usr", "/usr/."));
        assert!(!is_a_parent_of("/usr", "/usr/"));
        assert!(!is_a_parent_of("/usr", "/usr"));
        assert!(!is_a_parent_of("/usr/", "/usr"));
        assert!(!is_a_parent_of("/usr/andy", "/usr"));
        assert!(!is_a_parent_of("/usr/andy", "/usr/sibling"));
        assert!(!is_a_parent_of("/usr/folder", "/usr/folder_not_a_child"));
    }

    #[test]
    fn test_is_a_parent_of_root() {
        assert!(is_a_parent_of("/", "/usr/andy"));
        assert!(is_a_parent_of("/", "/usr"));
        assert!(!is_a_parent_of("/", "/"));
    }

    #[test]
    fn test_should_ignore_file() {
        let mut files = HashSet::new();
        files.insert((10, 20));

        assert!(!should_ignore_file(true, &mut files, Some((0, 0))));

        // New file is not known it will be inserted to the hashmp and should not be ignored
        assert!(!should_ignore_file(false, &mut files, Some((11, 12))));
        assert!(files.contains(&(11, 12)));

        // The same file will be ignored the second time
        assert!(should_ignore_file(false, &mut files, Some((11, 12))));
    }

    #[test]
    fn test_should_ignore_file_on_different_device() {
        let mut files = HashSet::new();
        files.insert((10, 20));

        // We do not ignore files on the same device
        assert!(!should_ignore_file(false, &mut files, Some((2, 99))));
        assert!(!should_ignore_file(true, &mut files, Some((2, 99))));
    }
}
