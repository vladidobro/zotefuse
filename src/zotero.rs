use crate::symlinkfs::SymlinkFS;
use crate::symlinkfs::Entry;
use std::collections::HashMap;
use sqlite;

pub fn zoterofs(path: &str) -> SymlinkFS {
    let fs = SymlinkFS(HashMap::from([
        (1, Entry::Dir(HashMap::from([("dir".to_string(), 2)]))),
        (2, Entry::Dir(HashMap::from([("link".to_string(), 3)]))),
        (3, Entry::Link("/Users/vladislavwohlrath/a".to_string())),
    ]));
    fs
}

pub fn read_db(path: &str) {
    let connection = sqlite::open(path).unwrap();
    let query = "SELECT * FROM collections WHERE id = ?";

    for row in connection
        .prepare(query)
        .unwrap()
        .into_iter()
        .bind((0, 1))
        .unwrap()
        .map(|row| row.unwrap())
    {
        println!("name = {}", row.read::<&str, _>("name"));
    }
}
