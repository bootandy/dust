use jwalk::DirEntry;

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

#[cfg(not(target_family = "unix"))]
pub fn get_metadata(d: &DirEntry, _apparent: bool) -> Option<(u64, Option<(u64, u64)>)> {
    d.metadata
        .as_ref()
        .unwrap()
        .as_ref()
        .ok()
        .map(|md| (md.len(), None))
}
