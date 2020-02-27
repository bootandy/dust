use jwalk::DirEntry;
#[allow(unused_imports)]
use std::fs;
use std::io;
use std::path::Path;

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
        if use_apparent_size {
            (md.len(), Some((md.ino(), md.dev())))
        } else {
            (md.blocks() * get_block_size(), Some((md.ino(), md.dev())))
        }
    })
}

#[cfg(target_family = "windows")]
pub fn get_metadata(d: &DirEntry, _use_apparent_size: bool) -> Option<(u64, Option<(u64, u64)>)> {
    // On windows opening the file to get size, file ID and volume can be very
    // expensive because 1) it causes a few system calls, and more importantly 2) it can cause
    // windows defender to scan the file.
    // Therefore we try to avoid doing that for common cases, mainly those of
    // plain files:

    // The idea is to make do with the file size that we get from the OS for
    // free as part of iterating a folder. Therefore we want to make sure that
    // it makes sense to use that free size information:

    // Volume boundaries:
    // The user can ask us not to cross volume boundaries. If the DirEntry is a
    // plain file and not a reparse point or other non-trivial stuff, we assume
    // that the file is located on the same volume as the directory that
    // contains it.

    // File ID:
    // This optimization does deprive us of access to a file ID. As a
    // workaround, we just make one up that hopefully does not collide with real
    // file IDs.
    // Hard links: Unresolved. We don't get inode/file index, so hard links
    // count once for each link. Hopefully they are not too commonly in use on
    // windows.

    // Size:
    // We assume (naively?) that for the common cases the free size info is the
    // same as one would get by doing the expensive thing. Sparse, encrypted and
    // compressed files are not included in the common cases, as one can image
    // there being more than view on their size.

    // Savings in orders of magnitude in terms of time, io and cpu have been
    // observed on hdd, windows 10, some 100Ks files taking up some hundreds of
    // GBs:
    // Consistently opening the file: 30 minutes.
    // With this optimization:         8 sec.

    fn get_metadata_expensive(d: &DirEntry) -> Option<(u64, Option<(u64, u64)>)> {
        use winapi_util::file::information;
        use winapi_util::Handle;

        let h = Handle::from_path_any(d.path()).ok()?;
        let info = information(&h).ok()?;

        Some((
            info.file_size(),
            Some((info.file_index(), info.volume_serial_number())),
        ))
    }

    match d.metadata {
        Some(Ok(ref md)) => {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20u32;
            const FILE_ATTRIBUTE_READONLY: u32 = 0x1u32;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2u32;
            const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4u32;
            const FILE_ATTRIBUTE_NORMAL: u32 = 0x80u32;
            const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10u32;

            let attr_filtered = md.file_attributes()
                & !(FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM);
            if attr_filtered == FILE_ATTRIBUTE_ARCHIVE
                || attr_filtered == FILE_ATTRIBUTE_DIRECTORY
                || md.file_attributes() == FILE_ATTRIBUTE_NORMAL
            {
                Some((md.len(), None))
            } else {
                get_metadata_expensive(&d)
            }
        }
        _ => get_metadata_expensive(&d),
    }
}

#[cfg(target_family = "unix")]
pub fn get_filesystem<P: AsRef<Path>>(file_path: P) -> Result<u64, io::Error> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata(file_path)?;
    Ok(metadata.dev())
}

#[cfg(target_family = "windows")]
pub fn get_filesystem<P: AsRef<Path>>(file_path: P) -> Result<u64, io::Error> {
    use winapi_util::file::information;
    use winapi_util::Handle;

    let h = Handle::from_path_any(file_path)?;
    let info = information(&h)?;
    Ok(info.volume_serial_number())
}
