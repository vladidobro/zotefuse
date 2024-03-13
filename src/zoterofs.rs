use fuser::{
    FileAttr, FileType, Filesystem, ReplyData, ReplyAttr, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use std::collections::HashMap;

pub struct ZoteroFS(pub HashMap<u64, Entry>);

pub enum Entry {
    Dir(HashMap<String, u64>),
    Link(String),
}

impl From<&Entry> for FileType {
    fn from(kind: &Entry) -> Self {
        match kind {
            Entry::Dir(_) => FileType::Directory,
            Entry::Link(_) => FileType::Symlink,
        }
    }
}

impl Entry {
    fn attr(&self, ino: u64) -> FileAttr {
        FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: self.into(),
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }
}

const TTL: Duration = Duration::from_secs(10);


impl Filesystem for ZoteroFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name = name.to_str().unwrap();
        println!("lookup {parent} {name}");
        let Some(parent_entry) = self.0.get(&parent) else { reply.error(ENOENT); return };
        match parent_entry {
            Entry::Link(_) => { reply.error(ENOENT); return },
            Entry::Dir(d) => {
                let Some(child_ino) = d.get(name) else { reply.error(ENOENT); return };
                let Some(child_entry) = self.0.get(child_ino) else { reply.error(ENOENT); return };
                reply.entry(&TTL, &(child_entry.attr(*child_ino)), 0);
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr {ino}");
        let Some(entry) = self.0.get(&ino) else { reply.error(ENOENT); return };
        reply.attr(&TTL, &(entry.attr(ino)));
    }

    fn readlink(
        &mut self, 
        _req: &Request<'_>, 
        ino: u64, 
        reply: ReplyData) 
    {
        println!("readlink {ino}");
        reply.error(ENOENT)
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir {ino}");
        let Some(entry) = self.0.get(&ino) else { reply.error(ENOENT); return };
        match entry {
            Entry::Link(_) => {
                println!("readdir link");
                reply.error(ENOENT);
                return
            },
            Entry::Dir(d) => {
                println!("readdir dir");
                let _ = reply.add(1, 1, FileType::Directory, ".");
                let _ = reply.add(1, 2, FileType::Directory, "..");
                for (i, (name, inode)) in d.iter().enumerate().skip(offset as usize) {
                    println!("readdir ino");
                    let res = match self.0.get(&inode).unwrap() {
                        Entry::Dir(_) => reply.add(*inode, (i + 3) as i64, FileType::Directory, &name),
                        Entry::Link(_) => reply.add(*inode, (i + 3) as i64, FileType::Symlink, &name),
                    };
                    if res { break; }
                }
                reply.ok();
            },
        }
    }
}
