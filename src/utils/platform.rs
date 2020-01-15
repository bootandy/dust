use jwalk::DirEntry;
use std::fs;

#[cfg(target_family = "unix")]
fn get_block_size() -> u64 {
    // All os specific implementations of MetatdataExt seem to define a block as 512 bytes
    // https://doc.rust-lang.org/std/os/linux/fs/trait.MetadataExt.html#tymethod.st_blocks
    512
}

#[cfg(target_family = "unix")]
pub fn get_metadata(d: &DirEntry, use_apparent_size: bool) -> Option<(u64, Option<(u64, u64)>)> {
    use std::os::unix::fs::MetadataExt;
    d.metadata.as_ref().unwrap().as_ref().ok().map(|md| {
        let inode = Some((md.ino(), md.dev()));
        if use_apparent_size {
            (md.len(), inode)
        } else {
            (md.blocks() * get_block_size(), inode)
        }
    })
}

#[cfg(target_family = "windows")]
pub fn get_metadata(d: &DirEntry, use_apparent_size: bool) -> Option<(u64, Option<(u64, u64)>)> {
    use std::os::windows::fs::MetadataExt;
    d.metadata.as_ref().unwrap().as_ref().ok().map(|md| {
        let windows_equivalent_of_inode = Some((md.file_index(), md.volume_serial_number()));
        (md.file_size(), windows_equivalent_of_inode)
    })
}

#[cfg(all(not(target_family = "windows"), not(target_family = "unix")))]
pub fn get_metadata(d: &DirEntry, _apparent: bool) -> Option<(u64, Option<(u64, u64)>)> {
    d.metadata
        .as_ref()
        .unwrap()
        .as_ref()
        .ok()
        .map(|md| (md.len(), None))
}

#[cfg(target_family = "unix")]
pub fn get_filesystem(file_path: &str) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata(file_path).unwrap();
    Some(metadata.dev())
}

#[cfg(target_family = "windows")]
pub fn get_device(file_path: &str) -> Option<u64> {
    use std::os::windows::fs::MetadataExt;
    let metadata = fs::metadata(file_path).unwrap();
    Some(metadata.volume_serial_number())
}

#[cfg(all(not(target_family = "windows"), not(target_family = "unix")))]
pub fn get_device(file_path: &str) -> Option<u64> {
    None
}
