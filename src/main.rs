use clap::Parser;
use fuser::MountOption;
use crate::symlinkfs::{SymlinkFS, entries_from_links};
use std::path::Path;

pub mod symlinkfs;
pub mod zotero;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    zoterohome: String,
    mountpoint: String,
}

fn main() {
    let cli = Cli::parse();

    let options = vec![
        MountOption::RO, 
        MountOption::FSName("zoterofs".to_string()), 
        MountOption::AutoUnmount
    ];

    let links = zotero::read_db(&cli.zoterohome);
    let storagepath = Path::new(&cli.zoterohome).join("storage");
    let entries = entries_from_links(links, storagepath.to_str().unwrap());
    let fs = SymlinkFS(entries);

    fuser::mount2(fs, &cli.mountpoint, &options).unwrap();
}
