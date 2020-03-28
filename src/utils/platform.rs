use ignore::DirEntry;
#[allow(unused_imports)]
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
    match d.metadata() {
        Ok(md) => {
            if use_apparent_size {
                Some((md.len(), Some((md.ino(), md.dev()))))
            } else {
                Some((md.blocks() * get_block_size(), Some((md.ino(), md.dev()))))
            }
        }
        Err(_e) => None,
    }
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

    use std::io;
    use std::path::Path;
    use winapi_util::Handle;
    fn handle_from_path_limited<P: AsRef<Path>>(path: P) -> io::Result<Handle> {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_READ_ATTRIBUTES: u32 = 0x0080;

        // So, it seems that it does does have to be that expensive to open
        // files to get their info: Avoiding opening the file with the full
        // GENERIC_READ is key:

        // https://docs.microsoft.com/en-us/windows/win32/secauthz/generic-access-rights:
        // "For example, a Windows file object maps the GENERIC_READ bit to the
        // READ_CONTROL and SYNCHRONIZE standard access rights and to the
        // FILE_READ_DATA, FILE_READ_EA, and FILE_READ_ATTRIBUTES
        // object-specific access rights"

        // The flag FILE_READ_DATA seems to be the expensive one, so we'll avoid
        // that, and a most of the other ones. Simply because it seems that we
        // don't need them.

        let file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .open(path)?;
        Ok(Handle::from_file(file))
    }

    fn get_metadata_expensive(d: &DirEntry) -> Option<(u64, Option<(u64, u64)>)> {
        use winapi_util::file::information;

        let h = handle_from_path_limited(d.path()).ok()?;
        let info = information(&h).ok()?;

        Some((
            info.file_size(),
            Some((info.file_index(), info.volume_serial_number())),
        ))
    }

    use std::os::windows::fs::MetadataExt;
    match d.metadata() {
        Ok(ref md) => {
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
