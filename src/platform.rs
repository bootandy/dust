#[allow(unused_imports)]
use std::fs;

use std::path::Path;

#[cfg(target_family = "unix")]
fn get_block_size() -> u64 {
    // All os specific implementations of MetadataExt seem to define a block as 512 bytes
    // https://doc.rust-lang.org/std/os/linux/fs/trait.MetadataExt.html#tymethod.st_blocks
    512
}

type InodeAndDevice = (u64, u64);
type FileTime = (i64, i64, i64);

/// The parsed stat fields the walker consumes.
pub type MetadataTuple = (u64, Option<InodeAndDevice>, FileTime);

#[cfg(target_family = "unix")]
pub fn get_metadata<P: AsRef<Path>>(
    path: P,
    use_apparent_size: bool,
    follow_links: bool,
) -> Option<MetadataTuple> {
    let metadata = if follow_links {
        path.as_ref().metadata()
    } else {
        path.as_ref().symlink_metadata()
    };
    match metadata {
        Ok(md) => tuple_from_metadata(&md, use_apparent_size),
        Err(_e) => None,
    }
}

/// Extract the data tuple from an already-fetched `Metadata`, no syscall.
#[cfg(target_family = "unix")]
pub fn tuple_from_metadata(
    md: &std::fs::Metadata,
    use_apparent_size: bool,
) -> Option<MetadataTuple> {
    use std::os::unix::fs::MetadataExt;
    let file_size = md.len();
    if use_apparent_size {
        Some((
            file_size,
            Some((md.ino(), md.dev())),
            (md.mtime(), md.atime(), md.ctime()),
        ))
    } else {
        // On NTFS mounts, the reported block count can be unexpectedly large.
        // To avoid overestimating disk usage, cap the allocated size to what the
        // file should occupy based on the file system I/O block size (blksize).
        // Related: https://github.com/bootandy/dust/issues/295
        let blksize = md.blksize();
        let target_size = file_size.div_ceil(blksize) * blksize;
        let reported_size = md.blocks() * get_block_size();

        // File systems can pre-allocate more space for a file than what would be necessary
        let pre_allocation_buffer = blksize * 65536;
        let max_size = target_size + pre_allocation_buffer;
        let allocated_size = if reported_size > max_size {
            target_size
        } else {
            reported_size
        };
        Some((
            allocated_size,
            Some((md.ino(), md.dev())),
            (md.mtime(), md.atime(), md.ctime()),
        ))
    }
}

#[cfg(target_family = "windows")]
fn filetime_to_unix_seconds(filetime: u64) -> i64 {
    const TICKS_PER_SECOND: i128 = 10_000_000;
    const UNIX_EPOCH_FILETIME: i128 = 116_444_736_000_000_000;

    ((i128::from(filetime) - UNIX_EPOCH_FILETIME).div_euclid(TICKS_PER_SECOND)) as i64
}

#[cfg(target_family = "windows")]
pub fn get_metadata<P: AsRef<Path>>(
    path: P,
    use_apparent_size: bool,
    follow_links: bool,
) -> Option<MetadataTuple> {
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
    use winapi_util::Handle;
    fn handle_from_path_limited(path: &Path) -> io::Result<Handle> {
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

    fn get_metadata_expensive(path: &Path, use_apparent_size: bool) -> Option<MetadataTuple> {
        use winapi_util::file::information;

        let h = handle_from_path_limited(path).ok()?;
        let info = information(&h).ok()?;

        if use_apparent_size {
            use filesize::PathExt;
            Some((
                path.size_on_disk().ok()?,
                Some((info.file_index(), info.volume_serial_number())),
                (
                    filetime_to_unix_seconds(info.last_write_time().unwrap()),
                    filetime_to_unix_seconds(info.last_access_time().unwrap()),
                    filetime_to_unix_seconds(info.creation_time().unwrap()),
                ),
            ))
        } else {
            Some((
                info.file_size(),
                Some((info.file_index(), info.volume_serial_number())),
                (
                    filetime_to_unix_seconds(info.last_write_time().unwrap()),
                    filetime_to_unix_seconds(info.last_access_time().unwrap()),
                    filetime_to_unix_seconds(info.creation_time().unwrap()),
                ),
            ))
        }
    }

    let path = path.as_ref();
    let metadata = if follow_links {
        path.metadata()
    } else {
        path.symlink_metadata()
    };
    match metadata {
        Ok(ref md) => tuple_from_metadata(md, use_apparent_size)
            .or_else(|| get_metadata_expensive(path, use_apparent_size)),
        _ => get_metadata_expensive(path, use_apparent_size),
    }
}

/// Extract the data tuple from an already-fetched `Metadata` on Windows.
/// Returns `None` when the file needs the expensive (handle-open) path —
/// the caller must fall back. Directories and normal files always return
/// `Some` (that's the cheap branch the current code already takes).
#[cfg(target_family = "windows")]
pub fn tuple_from_metadata(
    md: &std::fs::Metadata,
    use_apparent_size: bool,
) -> Option<MetadataTuple> {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;
    const FILE_ATTRIBUTE_READONLY: u32 = 0x01;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x04;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
    const FILE_ATTRIBUTE_SPARSE_FILE: u32 = 0x00000200;
    const FILE_ATTRIBUTE_PINNED: u32 = 0x00080000;
    const FILE_ATTRIBUTE_UNPINNED: u32 = 0x00100000;
    const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x00040000;
    const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x00400000;
    const FILE_ATTRIBUTE_OFFLINE: u32 = 0x00001000;
    // normally FILE_ATTRIBUTE_SPARSE_FILE would be enough, however Windows sometimes likes to mask it out. see: https://stackoverflow.com/q/54560454
    const IS_PROBABLY_ONEDRIVE: u32 = FILE_ATTRIBUTE_SPARSE_FILE
        | FILE_ATTRIBUTE_PINNED
        | FILE_ATTRIBUTE_UNPINNED
        | FILE_ATTRIBUTE_RECALL_ON_OPEN
        | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS
        | FILE_ATTRIBUTE_OFFLINE;
    let attr_filtered = md.file_attributes()
        & !(FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM);
    if ((attr_filtered & FILE_ATTRIBUTE_ARCHIVE) != 0
        || (attr_filtered & FILE_ATTRIBUTE_DIRECTORY) != 0
        || md.file_attributes() == FILE_ATTRIBUTE_NORMAL)
        && !((attr_filtered & IS_PROBABLY_ONEDRIVE != 0) && use_apparent_size)
    {
        Some((
            md.len(),
            None,
            (
                filetime_to_unix_seconds(md.last_write_time()),
                filetime_to_unix_seconds(md.last_access_time()),
                filetime_to_unix_seconds(md.creation_time()),
            ),
        ))
    } else {
        None
    }
}

/// What `--shared-blocks` records per file so blocks shared between files
/// (APFS clones) are counted once. By default: the clone id of a file that
/// shares all its blocks, which every member of a clone family has in common.
/// With the `exact-shared-blocks` feature: the file's physical extents, which
/// also catches partly rewritten clones.
#[cfg(not(feature = "exact-shared-blocks"))]
pub type SharedBlocks = Option<std::num::NonZeroU64>;
#[cfg(feature = "exact-shared-blocks")]
pub type SharedBlocks = Box<[(u64, u64)]>;

// From <sys/stat.h>; the libc crate doesn't define them.
#[cfg(all(target_os = "macos", feature = "exact-shared-blocks"))]
const EF_MAY_SHARE_BLOCKS: u64 = 0x1;
#[cfg(all(target_os = "macos", not(feature = "exact-shared-blocks")))]
const EF_SHARES_ALL_BLOCKS: u64 = 0x40;

/// The file's APFS clone id and extended flags (`EF_*`), from one getattrlist
/// call: no open, and it works on files we can't read. `None` when the
/// filesystem doesn't report them (EF_SHARES_ALL_BLOCKS needs macOS 14).
#[cfg(target_os = "macos")]
fn get_clone_attributes(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;

    #[repr(C)]
    struct Reply {
        length: u32,
        returned: libc::attribute_set_t,
        // The kernel packs attributes in bit order, not request order:
        // ATTR_CMNEXT_CLONEID (0x100) comes before ATTR_CMNEXT_EXT_FLAGS (0x200).
        clone_id: u64,
        ext_flags: u64,
    }

    let path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut request = libc::attrlist {
        bitmapcount: libc::ATTR_BIT_MAP_COUNT,
        reserved: 0,
        commonattr: libc::ATTR_CMN_RETURNED_ATTRS,
        volattr: 0,
        dirattr: 0,
        fileattr: 0,
        forkattr: libc::ATTR_CMNEXT_CLONEID | libc::ATTR_CMNEXT_EXT_FLAGS,
    };
    let mut reply: Reply = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::getattrlist(
            path.as_ptr(),
            (&raw mut request).cast(),
            (&raw mut reply).cast(),
            size_of::<Reply>(),
            libc::FSOPT_NOFOLLOW | libc::FSOPT_ATTR_CMN_EXTENDED,
        )
    };
    let wanted = request.forkattr;
    (result == 0 && reply.returned.forkattr & wanted == wanted)
        .then_some((reply.clone_id, reply.ext_flags))
}

#[cfg(all(target_os = "macos", not(feature = "exact-shared-blocks")))]
pub fn get_shared_blocks(path: &Path) -> SharedBlocks {
    let (clone_id, ext_flags) = get_clone_attributes(path)?;
    if ext_flags & EF_SHARES_ALL_BLOCKS == 0 {
        return None;
    }
    std::num::NonZeroU64::new(clone_id)
}

// Only files the kernel flags as possibly sharing blocks pay for the open
// and extent mapping; for the rest there's nothing to deduplicate.
#[cfg(all(target_os = "macos", feature = "exact-shared-blocks"))]
pub fn get_shared_blocks(path: &Path) -> SharedBlocks {
    match get_clone_attributes(path) {
        Some((_, ext_flags)) if ext_flags & EF_MAY_SHARE_BLOCKS != 0 => get_extents(path),
        _ => Box::default(),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_shared_blocks(_path: &Path) -> SharedBlocks {
    SharedBlocks::default()
}

/// A file's physical extents as `[start, end)` byte ranges on its device,
/// with each end rounded up to the filesystem's allocation block. Clones
/// report overlapping ranges. Returns an empty list when the file can't be
/// opened or fully mapped, so the caller counts it at its full size.
#[cfg(all(target_os = "macos", feature = "exact-shared-blocks"))]
fn get_extents(path: &Path) -> Box<[(u64, u64)]> {
    use std::os::fd::AsRawFd;

    let Ok(file) = fs::File::open(path) else {
        return Box::default();
    };
    let fd = file.as_raw_fd();
    let Ok(len) = file.metadata().map(|md| md.len()) else {
        return Box::default();
    };
    // f_frsize is the allocation block; f_bsize is only the preferred I/O size
    // and can be far larger (1 MiB on APFS).
    let mut vfs: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstatvfs(fd, &mut vfs) } != 0 || vfs.f_frsize == 0 {
        return Box::default();
    }
    let block = vfs.f_frsize as u64;

    let mut extents = Vec::new();
    let mut offset = 0;
    while offset < len {
        // In: file offset and bytes wanted. Out: device offset and how many
        // bytes from there are contiguous.
        let mut l2p = libc::log2phys {
            l2p_flags: 0,
            l2p_contigbytes: (len - offset) as libc::off_t,
            l2p_devoffset: offset as libc::off_t,
        };
        // Fails with ENOTSUP on compressed files.
        if unsafe { libc::fcntl(fd, libc::F_LOG2PHYS_EXT, &mut l2p) } == -1
            || l2p.l2p_contigbytes <= 0
        {
            return Box::default();
        }
        let contig = l2p.l2p_contigbytes as u64;
        // A hole (sparse file) maps to device offset -1 and has no blocks.
        if l2p.l2p_devoffset >= 0 {
            let start = l2p.l2p_devoffset as u64;
            extents.push((start, (start + contig).div_ceil(block) * block));
        }
        offset += contig;
    }
    extents.into_boxed_slice()
}
