use sqlite::Row;

const DEFAULT_QUERY: &str = "
with recursive
cte_attachments as (
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
),
cte_collectionitems as (
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
),
cte_collections AS (
        SELECT
        collectionID,
        collectionName,
        collectionName path
    FROM collections cl
    WHERE cl.parentCollectionID IS NULL

    UNION

    SELECT
        ch.collectionID,
        ch.collectionName,
        p.path || '/' || ch.collectionName path
    FROM collections ch, cte_collections p
    WHERE ch.parentCollectionID=p.collectionID
)

select 
    c.path || '/' ||  a.title as mountpath,
    a.path as storagepath
from cte_attachments as a
join cte_collectionitems as i
join cte_collections as c
on a.attachmentId = i.attachmentId
and c.collectionID = i.collectionID

";

pub fn read_db(zoterohome: &str) -> Vec<(String, String)>{
    let mut db_path = String::from(zoterohome);
    db_path.push_str("zotero.sqlite");
    let connection = sqlite::open(db_path).unwrap();

    let mut links = Vec::new();

    for row in connection
        .prepare(DEFAULT_QUERY)
        .unwrap()
        .iter()
        .map(|row| row.unwrap())
    {
        let (mountpath, storagepath) = row_to_paths(&row);
        links.push((String::from(mountpath), String::from(storagepath)));
    }

    links
}

pub fn row_to_paths(row: &Row) -> (&str, &str) {
    let mountpath = row.read::<&str, _>("mountpath");
    let storagepath = row.read::<&str, _>("storagepath");
    (mountpath, storagepath)
}
