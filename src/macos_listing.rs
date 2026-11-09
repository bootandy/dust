//! Folder listing with `getattrlistbulk(2)` on macOS.
//!
//! `fs::read_dir` and then one `lstat` per entry costs one system call and
//! one path lookup per entry. `getattrlistbulk` returns a batch of a
//! folder's entries together with their attributes. Apple's fts(3), and so
//! `du`, list folders this way.
//!
//! A regular file's `stat` fields come from the batch. Every other entry,
//! and a file the call returned an error or a partial record for, is
//! stat'ed by the walker as before. Folders always are: the batch describes
//! a mount point or a firmlink itself, where `lstat` describes what it leads
//! to.
//!
//! The walker lists a folder with `fs::read_dir` instead, as before, when
//! the call fails on it, when a record lacks its name, type or inode, or
//! when the folder's link count shows more than `MAX_ENTRIES` entries. Its
//! errors are then the ones dust reported before. A folder that can be
//! listed but not searched fails the call with `EACCES`.
//!
//! On ExFAT an empty file has no cluster and so no lasting inode number:
//! each listing or lookup gives it a new one, and the batch's differs from
//! `lstat`'s. They are unique within a walk, so hard-link dedup is
//! unaffected; `-f` already takes `d_ino` from `read_dir` the same way.

use std::cell::RefCell;
use std::ffi::{CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::platform::StatFields;

// <sys/attr.h> and <sys/vnode.h>; the libc crate does not define these.
const ATTR_CMN_ERROR: libc::attrgroup_t = 0x2000_0000;
const VREG: u32 = 1;
const VDIR: u32 = 2;
const VLNK: u32 = 5;

const COMMON_ATTRS: libc::attrgroup_t = libc::ATTR_CMN_RETURNED_ATTRS
    | ATTR_CMN_ERROR
    | libc::ATTR_CMN_NAME
    | libc::ATTR_CMN_DEVID
    | libc::ATTR_CMN_OBJTYPE
    | libc::ATTR_CMN_MODTIME
    | libc::ATTR_CMN_CHGTIME
    | libc::ATTR_CMN_ACCTIME
    | libc::ATTR_CMN_FILEID;
const FILE_ATTRS: libc::attrgroup_t =
    libc::ATTR_FILE_ALLOCSIZE | libc::ATTR_FILE_IOBLOCKSIZE | libc::ATTR_FILE_DATALENGTH;

/// Bytes per call: room for several hundred entries.
const BUF_LEN: usize = 128 * 1024;

/// On APFS and HFS+ a folder's link count is its entry count plus 2, up to
/// 65,535. A folder whose count shows more entries than this is left to
/// `fs::read_dir`, so the walker stats its files in parallel. The calls for
/// one folder run on one thread, and on APFS their cost per entry grows
/// with the folder: about 1.3 us of system time per entry at 30,000 entries
/// and 9 us at 1,000,000. There the listing took about twice the wall time
/// of the parallel stat, though a quarter of its CPU time.
///
/// ExFAT reports a link count of 1, so its folders are listed in bulk at
/// any size. On an ExFAT image whose largest folder holds 62,000 entries,
/// the walk took about 60% of the wall time and a fifth of the CPU time of
/// the parallel stat.
const MAX_ENTRIES: usize = 50_000;

thread_local! {
    // One buffer per walker thread, reused for every folder it lists.
    static BUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy)]
pub enum Kind {
    File,
    Dir,
    Symlink,
    Other,
}

pub struct Entry {
    /// The entry's name in its folder.
    pub name: Box<OsStr>,
    pub kind: Kind,
    pub ino: u64,
    /// A regular file's `stat` fields, when the call returned all of them.
    pub stat: Option<StatFields>,
}

/// Lists `dir`, whose link count is `nlink`. `None` means the caller should
/// use `fs::read_dir` instead.
pub fn read_dir(dir: &Path, nlink: u64) -> Option<Vec<Entry>> {
    if nlink > MAX_ENTRIES as u64 + 2 {
        return None;
    }
    BUF.with_borrow_mut(|buf| list(dir, buf))
}

/// Closes the folder's descriptor when the listing is done.
struct Fd(libc::c_int);

impl Drop for Fd {
    fn drop(&mut self) {
        // SAFETY: the descriptor is ours, open, and closed only here.
        unsafe { libc::close(self.0) };
    }
}

fn list(dir: &Path, buf: &mut Vec<u8>) -> Option<Vec<Entry>> {
    let c_dir = CString::new(dir.as_os_str().as_bytes()).ok()?;
    // The flags opendir(3) uses.
    // SAFETY: open reads the NUL-terminated path, which outlives the call.
    let fd = unsafe {
        libc::open(
            c_dir.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return None;
    }
    let fd = Fd(fd);
    let mut attrs = libc::attrlist {
        bitmapcount: libc::ATTR_BIT_MAP_COUNT,
        reserved: 0,
        commonattr: COMMON_ATTRS,
        volattr: 0,
        dirattr: 0,
        fileattr: FILE_ATTRS,
        forkattr: 0,
    };
    buf.resize(BUF_LEN, 0);
    let mut entries = Vec::new();
    loop {
        // SAFETY: the call reads `attrs` and writes at most `buf.len()`
        // bytes into `buf`; both are ours and outlive it.
        let count = unsafe {
            libc::getattrlistbulk(
                fd.0,
                (&raw mut attrs).cast(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                0,
            )
        };
        // Any failure, on the first call or a later one, hands the whole
        // folder to read_dir, which reports what it reported before.
        if count < 0 {
            return None;
        }
        if count == 0 {
            return Some(entries);
        }
        let mut at = 0;
        for _ in 0..count {
            let (record, len) = parse(buf.get(at..)?)?;
            at += len;
            // read_dir skips these two, and the walk must never descend them.
            if record.name == b"." || record.name == b".." {
                continue;
            }
            let kind = match record.objtype? {
                VREG => Kind::File,
                VDIR => Kind::Dir,
                VLNK => Kind::Symlink,
                _ => Kind::Other,
            };
            entries.push(Entry {
                name: OsStr::from_bytes(record.name).into(),
                kind,
                ino: record.ino?,
                stat: record.stat(),
            });
        }
    }
}

/// One record of a batch: the attributes it returned.
struct Record<'a> {
    name: &'a [u8],
    error: u32,
    dev: Option<i32>,
    objtype: Option<u32>,
    mtime: Option<i64>,
    ctime: Option<i64>,
    atime: Option<i64>,
    ino: Option<u64>,
    alloc: Option<i64>,
    iosize: Option<u32>,
    len: Option<i64>,
}

impl Record<'_> {
    /// The fields `lstat` would return for a regular file, when the batch
    /// returned all of them without an error.
    fn stat(&self) -> Option<StatFields> {
        if self.error != 0 || self.objtype? != VREG {
            return None;
        }
        let blksize = self.iosize?;
        // A zero block size would divide by zero in `tuple_from_stat`;
        // leave such a file to `lstat`.
        if blksize == 0 {
            return None;
        }
        Some(StatFields {
            len: u64::try_from(self.len?).ok()?,
            // As xnu's vn_stat, and Apple's fts(3), compute st_blocks.
            blocks: u64::try_from(self.alloc?).ok()?.div_ceil(512),
            blksize: u64::from(blksize),
            ino: self.ino?,
            // As std's MetadataExt::dev widens st_dev.
            dev: self.dev? as u64,
            mtime: self.mtime?,
            atime: self.atime?,
            ctime: self.ctime?,
        })
    }
}

fn take<const N: usize>(rec: &[u8], at: &mut usize) -> Option<[u8; N]> {
    let bytes = rec.get(*at..at.checked_add(N)?)?;
    *at += N;
    bytes.try_into().ok()
}

fn take_u32(rec: &[u8], at: &mut usize) -> Option<u32> {
    take(rec, at).map(u32::from_ne_bytes)
}

fn take_i64(rec: &[u8], at: &mut usize) -> Option<i64> {
    take(rec, at).map(i64::from_ne_bytes)
}

/// The seconds of a `struct timespec`; dust reads whole seconds only.
fn take_seconds(rec: &[u8], at: &mut usize) -> Option<i64> {
    let sec = take_i64(rec, at)?;
    let _nsec = take_i64(rec, at)?;
    Some(sec)
}

/// Reads the record at the start of `buf`, and returns it with its length.
/// The layout is getattrlist(2)'s: the length, the returned attribute set,
/// the error code, then each returned attribute in bit order, every one a
/// multiple of 4 bytes long.
fn parse(buf: &[u8]) -> Option<(Record<'_>, usize)> {
    let len = take_u32(buf, &mut 0)? as usize;
    let rec = buf.get(..len)?;
    let mut at = 4;
    let common = take_u32(rec, &mut at)?;
    let _vol = take_u32(rec, &mut at)?;
    let _dir = take_u32(rec, &mut at)?;
    let file = take_u32(rec, &mut at)?;
    let _fork = take_u32(rec, &mut at)?;
    let mut record = Record {
        name: &[],
        error: 0,
        dev: None,
        objtype: None,
        mtime: None,
        ctime: None,
        atime: None,
        ino: None,
        alloc: None,
        iosize: None,
        len: None,
    };
    if common & ATTR_CMN_ERROR != 0 {
        record.error = take_u32(rec, &mut at)?;
    }
    if common & libc::ATTR_CMN_NAME == 0 {
        return None;
    }
    // An attrreference_t: the name's offset from this field, and its length
    // with the terminating NUL.
    let field = at;
    let offset = i32::from_ne_bytes(take(rec, &mut at)?);
    let name_len = take_u32(rec, &mut at)? as usize;
    let start = field.checked_add_signed(isize::try_from(offset).ok()?)?;
    let name = rec.get(start..start.checked_add(name_len)?)?;
    record.name = CStr::from_bytes_until_nul(name).ok()?.to_bytes();
    if record.name.is_empty() {
        return None;
    }
    if common & libc::ATTR_CMN_DEVID != 0 {
        record.dev = Some(i32::from_ne_bytes(take(rec, &mut at)?));
    }
    if common & libc::ATTR_CMN_OBJTYPE != 0 {
        record.objtype = Some(take_u32(rec, &mut at)?);
    }
    if common & libc::ATTR_CMN_MODTIME != 0 {
        record.mtime = Some(take_seconds(rec, &mut at)?);
    }
    if common & libc::ATTR_CMN_CHGTIME != 0 {
        record.ctime = Some(take_seconds(rec, &mut at)?);
    }
    if common & libc::ATTR_CMN_ACCTIME != 0 {
        record.atime = Some(take_seconds(rec, &mut at)?);
    }
    if common & libc::ATTR_CMN_FILEID != 0 {
        record.ino = Some(u64::from_ne_bytes(take(rec, &mut at)?));
    }
    if file & libc::ATTR_FILE_ALLOCSIZE != 0 {
        record.alloc = Some(take_i64(rec, &mut at)?);
    }
    if file & libc::ATTR_FILE_IOBLOCKSIZE != 0 {
        record.iosize = Some(take_u32(rec, &mut at)?);
    }
    if file & libc::ATTR_FILE_DATALENGTH != 0 {
        record.len = Some(take_i64(rec, &mut at)?);
    }
    Some((record, len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    #[test]
    fn test_bulk_listing_matches_read_dir_and_lstat() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("empty"), b"").unwrap();
        fs::write(dir.join("small"), b"hello").unwrap();
        fs::write(dir.join("large"), vec![7u8; 1 << 20]).unwrap();
        fs::create_dir(dir.join("sub")).unwrap();
        std::os::unix::fs::symlink("small", dir.join("link")).unwrap();
        fs::hard_link(dir.join("small"), dir.join("hard")).unwrap();
        fs::write(dir.join("ラウトは難しいです！.japan"), b"unicode").unwrap();
        // More entries than one buffer holds, so the folder takes several calls.
        for i in 0..3000 {
            fs::write(dir.join(format!("f{i}")), b"x").unwrap();
        }

        let expected: HashMap<std::ffi::OsString, fs::Metadata> = fs::read_dir(dir)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                let md = fs::symlink_metadata(e.path()).unwrap();
                (e.file_name(), md)
            })
            .collect();
        let nlink = fs::metadata(dir).unwrap().nlink();
        let listed = read_dir(dir, nlink).expect("getattrlistbulk lists a temp folder");
        // A link count over the limit leaves the folder to read_dir.
        assert!(read_dir(dir, MAX_ENTRIES as u64 + 3).is_none());

        assert_eq!(listed.len(), expected.len());
        for entry in &listed {
            let md = &expected[&*entry.name];
            let file_type = md.file_type();
            assert_eq!(entry.ino, md.ino(), "{:?}", entry.name);
            assert_eq!(matches!(entry.kind, Kind::File), file_type.is_file());
            assert_eq!(matches!(entry.kind, Kind::Dir), file_type.is_dir());
            assert_eq!(matches!(entry.kind, Kind::Symlink), file_type.is_symlink());
            match entry.stat {
                Some(st) => {
                    assert!(file_type.is_file(), "{:?}", entry.name);
                    assert_eq!(
                        (st.len, st.blocks, st.blksize, st.ino, st.dev),
                        (md.len(), md.blocks(), md.blksize(), md.ino(), md.dev()),
                        "{:?}",
                        entry.name
                    );
                    assert_eq!(
                        (st.mtime, st.atime, st.ctime),
                        (md.mtime(), md.atime(), md.ctime()),
                        "{:?}",
                        entry.name
                    );
                }
                None => assert!(!file_type.is_file(), "{:?}", entry.name),
            }
        }
    }

    #[test]
    fn test_bulk_listing_leaves_unsearchable_folder_to_read_dir() {
        // A folder that can be listed but not searched fails getattrlistbulk
        // with EACCES once more than names and types are requested.
        let tmp = tempfile::tempdir().unwrap();
        let locked = tmp.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::write(locked.join("file"), b"x").unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o444)).unwrap();

        let listed = read_dir(&locked, 3);
        let names = fs::read_dir(&locked).map(|it| it.count());
        // Root can search it anyway; then there is nothing to observe.
        let searchable = fs::symlink_metadata(locked.join("file")).is_ok();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

        if !searchable {
            assert!(listed.is_none());
            assert_eq!(names.unwrap(), 1);
        }
    }
}
