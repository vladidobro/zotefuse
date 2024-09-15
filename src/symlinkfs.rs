use fuser::{
    FileAttr, FileType, Filesystem, ReplyData, ReplyAttr, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
use std::fs::canonicalize;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use std::collections::HashMap;
use std::path::{Path, Component};

pub struct SymlinkFS(pub HashMap<u64, Entry>);


pub fn entries_from_links(links: Vec<(String, String)>, basedir: &str) -> HashMap<u64, Entry> {
    let basedir = canonicalize(Path::new(basedir)).unwrap();

    let mut entries = HashMap::new();
    entries.insert(1, Entry::Dir(HashMap::new()));
    let mut curinode = 2;

    for (mountpoint, storagepath) in links.iter() {
        entries.insert(
            curinode, 
            Entry::Link(
                String::from(
                    basedir
                    .join(storagepath)
                    .to_str()
                    .unwrap()
                )
            )
        );
        let linkinode = curinode;
        curinode += 1;

        let mnt = Path::new(mountpoint);

        let mut dirinode: u64 = 1;
        if let Some(parent) = mnt.parent() {
            for component in parent.components() {
                let Component::Normal(name) = component else { panic!() };
                let name = String::from(name.to_str().unwrap());
                let Entry::Dir(dirmap) = entries.get_mut(&dirinode).unwrap() else { panic!() };
                if !dirmap.contains_key(&name) {
                    dirmap.insert(name, curinode);
                    entries.insert(curinode, Entry::Dir(HashMap::new()));
                    dirinode = curinode;
                    curinode += 1;
                } else {
                    dirinode = *dirmap.get(&name).unwrap();
                }
            }
        }
        let Some(Entry::Dir(dirmap)) = entries.get_mut(&dirinode) else { panic!() };
        let name = mnt.file_name().unwrap();
        dirmap.insert(String::from(name.to_str().unwrap()), linkinode);
    }

    entries
}

#[derive(Debug)]
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


impl Filesystem for SymlinkFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name = name.to_str().unwrap();
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
        let Some(entry) = self.0.get(&ino) else { reply.error(ENOENT); return };
        reply.attr(&TTL, &(entry.attr(ino)));
    }

    fn readlink(
        &mut self, 
        _req: &Request<'_>, 
        ino: u64, 
        reply: ReplyData) 
    {
        let Some(entry) = self.0.get(&ino) else { reply.error(ENOENT); return };
        match entry {
            Entry::Dir(_) => {
                reply.error(ENOENT);
                return;
            }
            Entry::Link(l) => {
                reply.data(l.as_bytes())
            }
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let Some(entry) = self.0.get(&ino) else { reply.error(ENOENT); return };
        match entry {
            Entry::Link(_) => {
                reply.error(ENOENT);
                return
            },
            Entry::Dir(d) => {
                let s = ".".to_string();  // :(
                let ss = "..".to_string(); // what do?

                let dots = vec![
                    (ino, FileType::Directory, &s),
                    (ino, FileType::Directory, &ss),
                ];
                let entries = d.iter().map(|e| {
                    let entry = self.0.get(&e.1).unwrap();
                    (*e.1, FileType::from(entry), e.0)
                });
                for (i, (ino, ftype, name)) in dots.into_iter().chain(entries).enumerate().skip(offset as usize) {
                    if reply.add(ino, (i + 3) as i64, ftype, &name) {
                        break;
                    }
                }
                reply.ok()
            },
        }
    }
}
