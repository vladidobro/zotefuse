use clap::{Arg, ArgAction, Command};
use fuser::MountOption;
use std::collections::HashMap;
use crate::symlinkfs::{SymlinkFS, Entry};
use crate::zotero::zoterofs;

mod symlinkfs;
mod zotero;

fn main() {
    let matches = Command::new("zoterofs")
        .author("Vladislav Wohlrath")
        .arg(
            Arg::new("MOUNTPOINT")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("ZOTERODIR")
                .required(true)
                .index(2)
        )
        .arg(
            Arg::new("no-auto-unmount")
                .long("no-auto-unmount")
                .action(ArgAction::SetFalse)
                .help("Don't automatically unmount on process exit"),
        )
        .get_matches();
    let mountpoint = matches.get_one::<String>("MOUNTPOINT").unwrap();
    let zoterodir = matches.get_one::<String>("ZOTERODIR").unwrap();
    let mut options = vec![MountOption::RO, MountOption::FSName("zoterofs".to_string())];
    if !matches.get_flag("no-auto-unmount") {
        options.push(MountOption::AutoUnmount);
    }

    //let fs = zoterofs(&zoterodir);


    zotero::read_db(&zoterodir);
    //fuser::mount2(fs, mountpoint, &options).unwrap();
}
