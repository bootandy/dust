use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use channel::Receiver;
use std::thread::JoinHandle;

use ignore::{WalkBuilder, WalkState};
use std::sync::atomic;
use std::thread;

mod platform;
use self::platform::*;

type PathData = (PathBuf, u64, Option<(u64, u64)>);

#[derive(Debug, Default, Eq)]
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
    max_depth: Option<usize>,
) -> WalkBuilder {
    let mut it = top_level_names.iter();
    let mut builder = WalkBuilder::new(it.next().unwrap());
    builder.follow_links(false);
    builder.ignore(false);
    builder.git_global(false);
    builder.git_ignore(false);
    builder.git_exclude(false);
    builder.hidden(false);

    if limit_filesystem {
        builder.same_file_system(true);
    }

    builder.max_depth(max_depth);

    for b in it {
        builder.add(b);
    }
    builder
}

pub fn get_dir_tree<P: AsRef<Path>>(
    top_level_names: &HashSet<P>,
    ignore_directories: &Option<Vec<PathBuf>>,
    apparent_size: bool,
    limit_filesystem: bool,
    by_filecount: bool,
    max_depth: Option<usize>,
) -> (bool, HashMap<PathBuf, u64>) {
    let (tx, rx) = channel::bounded::<PathData>(1000);

    let permissions_flag = AtomicBool::new(true);

    let t2 = HashSet::from_iter(top_level_names.iter().map(|p| p.as_ref().to_path_buf()));

    let t = create_reader_thread(rx, t2, apparent_size);
    let walk_dir_builder = prepare_walk_dir_builder(top_level_names, limit_filesystem, max_depth);

    walk_dir_builder.build_parallel().run(|| {
        let txc = tx.clone();
        let pf = &permissions_flag;
        Box::new(move |path| {
            match path {
                Ok(p) => {
                    if let Some(dirs) = ignore_directories {
                        let path = p.path();
                        let parts = path.components().collect::<Vec<std::path::Component>>();
                        for d in dirs {
                            let seq = d.components().collect::<Vec<std::path::Component>>();
                            if parts
                                .windows(seq.len())
                                .any(|window| window.iter().collect::<PathBuf>() == *d)
                            {
                                return WalkState::Continue;
                            }
                        }
                    }

                    let maybe_size_and_inode = get_metadata(&p, apparent_size);

                    match maybe_size_and_inode {
                        Some(data) => {
                            let (mut size, inode_device) = data;
                            if by_filecount {
                                size = 1;
                            }
                            txc.send((p.into_path(), size, inode_device)).unwrap();
                        }
                        None => {
                            pf.store(false, atomic::Ordering::Relaxed);
                        }
                    }
                }
                Err(_) => {
                    pf.store(false, atomic::Ordering::Relaxed);
                }
            };
            WalkState::Continue
        })
    });

    drop(tx);
    let data = t.join().unwrap();
    (permissions_flag.load(atomic::Ordering::SeqCst), data)
}

fn create_reader_thread(
    rx: Receiver<PathData>,
    top_level_names: HashSet<PathBuf>,
    apparent_size: bool,
) -> JoinHandle<HashMap<PathBuf, u64>> {
    // Receiver thread
    thread::spawn(move || {
        let mut hash: HashMap<PathBuf, u64> = HashMap::new();
        let mut inodes: HashSet<(u64, u64)> = HashSet::new();

        for dent in rx {
            let (path, size, maybe_inode_device) = dent;

            if should_ignore_file(apparent_size, &mut inodes, maybe_inode_device) {
                continue;
            } else {
                for p in path.ancestors() {
                    let s = hash.entry(p.to_path_buf()).or_insert(0);
                    *s += size;

                    if top_level_names.contains(p) {
                        break;
                    }
                }
            }
        }
        hash
    })
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
