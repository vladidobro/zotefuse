use crate::symlinkfs::SymlinkFS;
use crate::symlinkfs::Entry;
use std::collections::HashMap;
use sqlite::State;

const LINKS_OFFSET: u64 = 2^22;

const QUERY_COL: &str = "
    select
        collectionID collectionId,
        parentCollectionID parentCollectionId,
        collectionName
    from collections
    ;
";

const QUERY_ATT: &str = "
    select 
        a.itemId attachmentId,
        (i.key || '/' || substr(a.path, 9)) path,
        d.title title,
        cr.firstName firstName,
        cr.lastName lastName,
        cr.numberOfAuthors numberOfAuthors
    from itemAttachments a
    left join items i on i.itemId = a.itemId
    left join items p on p.itemId = coalesce(a.parentItemId, a.itemId)
    left join (
        select 
            ficr.itemId itemId,
            fcr.firstName firstName,
            fcr.lastName lastName,
            nicr.numberOfAuthors numberOfAuthors
        from itemCreators ficr
        left join creators fcr on fcr.creatorId = ficr.creatorId
        left join (
            select
                itemId,
                count(*) numberOfAuthors
            from itemCreators 
            group by itemId
        ) nicr on nicr.itemId = ficr.itemId
        where ficr.orderIndex = 0
    ) cr on cr.itemId = p.itemId
    left join (
        select
            id.itemId itemId,
            idv.value title
        from itemData id
        left join fields f on f.fieldId = id.fieldId
        left join itemDataValues idv on idv.valueId = id.valueId
        where f.fieldName = 'title'
    ) d on d.itemId = p.itemId
    where a.path like 'storage:%'
    ;
";

const QUERY_COLATT: &str = "
    select 
        c.collectionID collectionId,
        a.attachmentId attachmentId
    from collectionItems c
    join (
        select 
            itemId attachmentId,
            coalesce(parentItemId, itemId) parentItemId
        from itemAttachments
    ) a on a.parentItemId = c.itemId
    ;
";

pub fn zoterofs(path: &str) -> SymlinkFS {
    let fs = SymlinkFS(HashMap::from([
        (1, Entry::Dir(HashMap::from([("dir".to_string(), 2)]))),
        (2, Entry::Dir(HashMap::from([("link".to_string(), 3)]))),
        (3, Entry::Link("/Users/vladislavwohlrath/a".to_string())),
    ]));
    fs
}

fn get_filename() {

}

pub fn read_db(path: &str) {
    let connection = sqlite::open(path).unwrap();

    let mut statement = connection.prepare(QUERY_ATT).unwrap();

    while let Ok(State::Row) = statement.next() {
        println!("name = {}", statement.read::<String, _>("lastName").unwrap());
    }
}
