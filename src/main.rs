use clap::{Arg, ArgAction, Command};
use fuser::{
    FileAttr, FileType, Filesystem, ReplyData, MountOption, ReplyAttr, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use std::collections::HashMap;


enum Inode {
    Dir(Dir),
    Link(Link),
}

struct Link {
    inode: u64,
    target: String,
}

struct Dir {
    inode: u64,
    contents: HashMap<String, u64>,
}

impl Inode {
    fn attr(&self) -> FileAttr {
        match self {
            Inode::Dir(d) => FileAttr {
                ino: d.inode,
                size: 0,
                blocks: 0,
                atime: UNIX_EPOCH,
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 2,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
                blksize: 512,
            },
            Inode::Link(l) => FileAttr {
                ino: l.inode,
                size: 0,
                blocks: 0,
                atime: UNIX_EPOCH,
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: FileType::Symlink,
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
}

const TTL: Duration = Duration::from_secs(1); // 1 second

struct ZoteroFS {
    inodes: HashMap<u64, Inode>
}

impl Filesystem for ZoteroFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.inodes.get(&parent) {
            Some(i) => match i {
                Inode::Dir(d) => {
                    match name.to_str() {
                        Some(n) => match d.contents.get(n) {
                            Some(i) => match self.inodes.get(&i) {
                                Some(inode) => reply.entry(&TTL, &(inode.attr()), 0),
                                None => reply.error(ENOENT),
                            },
                            None => reply.error(ENOENT),
                        },
                        None => reply.error(ENOENT),
                    }
                }
                Inode::Link(l) => reply.error(ENOENT),
            },
            None => reply.error(ENOENT),
        };
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.inodes.get(&ino) {
            Some(i) => reply.attr(&TTL, &(i.attr())),
            None => reply.error(ENOENT)
        }
    }

    fn readlink(
        &mut self, 
        _req: &Request<'_>, 
        ino: u64, 
        reply: ReplyData) 
    {
        // TODO
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        match self.inodes.get(&ino) {
            Some(i) => match i {
                Inode::Dir(d) => {
                    reply.add(1, 1, FileType::Directory, ".");
                    reply.add(1, 2, FileType::Directory, "..");
                    for (name, inode) in self.inodes {
                        match inode {
                            Inode::Dir(d) => reply.add(d.inode, d.inode, FileType::Directory, name),
                            Inode::Link(d) => reply.add(d.inode, d.inode, FileType::Symlink, name),
                        };
                    }
                },
                Inode::Link(l) => reply.error(ENOENT),
            }
            None => reply.error(ENOENT),
        }
        reply.ok();
    }
}

fn main() {
    let matches = Command::new("hello")
        .author("Christopher Berner")
        .arg(
            Arg::new("MOUNT_POINT")
                .required(true)
                .index(1)
                .help("Act as a client, and mount FUSE at given path"),
        )
        .arg(
            Arg::new("auto_unmount")
                .long("auto_unmount")
                .action(ArgAction::SetTrue)
                .help("Automatically unmount on process exit"),
        )
        .arg(
            Arg::new("allow-root")
                .long("allow-root")
                .action(ArgAction::SetTrue)
                .help("Allow root user to access filesystem"),
        )
        .get_matches();
    let mountpoint = matches.get_one::<String>("MOUNT_POINT").unwrap();
    let mut options = vec![MountOption::RO, MountOption::FSName("hello".to_string())];
    if matches.get_flag("auto_unmount") {
        options.push(MountOption::AutoUnmount);
    }
    if matches.get_flag("allow-root") {
        options.push(MountOption::AllowRoot);
    }
    fuser::mount2(ZoteroFS, mountpoint, &options).unwrap();
}
