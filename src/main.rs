use clap::{Arg, ArgAction, Command};
use fuser::MountOption;
use std::collections::HashMap;
use crate::zoterofs::{ZoteroFS, Entry};

mod zoterofs;

fn main() {
    let matches = Command::new("zoterofs")
        .author("Vladislav Wohlrath")
        .arg(
            Arg::new("MOUNTPOINT")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("no-auto-unmount")
                .long("no-auto-unmount")
                .action(ArgAction::SetFalse)
                .help("Don't automatically unmount on process exit"),
        )
        .get_matches();
    let mountpoint = matches.get_one::<String>("MOUNTPOINT").unwrap();
    let mut options = vec![MountOption::RO, MountOption::FSName("zoterofs".to_string())];
    if !matches.get_flag("no-auto-unmount") {
        options.push(MountOption::AutoUnmount);
    }

    let fs = ZoteroFS(HashMap::from([
        (1, Entry::Dir(HashMap::from([("dir".to_string(), 2)]))),
        (2, Entry::Dir(HashMap::new())),
    ]));

    fuser::mount2(fs, mountpoint, &options).unwrap();
}
