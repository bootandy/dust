use std;

fn get_block_size() -> u64 {
    // All os specific implementations of MetatdataExt seem to define a block as 512 bytes
    // https://doc.rust-lang.org/std/os/linux/fs/trait.MetadataExt.html#tymethod.st_blocks
    512
}

#[cfg(target_os = "linux")]
pub fn get_metadata(
    d: &std::fs::DirEntry,
    use_apparent_size: bool,
) -> Option<(u64, Option<(u64, u64)>)> {
    use std::os::linux::fs::MetadataExt;
    match d.metadata().ok() {
        Some(md) => {
            let inode = Some((md.st_ino(), md.st_dev()));
            if use_apparent_size {
                Some((md.len(), inode))
            } else {
                Some((md.st_blocks() * get_block_size(), inode))
            }
        }
        None => None,
    }
}

#[cfg(target_os = "unix")]
pub fn get_metadata(
    d: &std::fs::DirEntry,
    use_apparent_size: bool,
) -> Option<(u64, Option<(u64, u64)>)> {
    use std::os::unix::fs::MetadataExt;
    match d.metadata().ok() {
        Some(md) => {
            let inode = Some((md.ino(), md.dev()));
            if use_apparent_size {
                Some((md.len(), inode))
            } else {
                Some((md.blocks() * get_block_size(), inode))
            }
        }
        None => None,
    }
}

#[cfg(target_os = "macos")]
pub fn get_metadata(
    d: &std::fs::DirEntry,
    use_apparent_size: bool,
) -> Option<(u64, Option<(u64, u64)>)> {
    use std::os::macos::fs::MetadataExt;
    match d.metadata().ok() {
        Some(md) => {
            let inode = Some((md.st_ino(), md.st_dev()));
            if use_apparent_size {
                Some((md.len(), inode))
            } else {
                Some((md.st_blocks() * get_block_size(), inode))
            }
        }
        None => None,
    }
}

#[cfg(not(any(target_os = "linux", target_os = "unix", target_os = "macos")))]
pub fn get_metadata(d: &std::fs::DirEntry, _apparent: bool) -> Option<(u64, Option<(u64, u64)>)> {
    match d.metadata().ok() {
        Some(md) => Some((md.len(), None)),
        None => None,
    }
}
