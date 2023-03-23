CREATE TEMPORARY VIEW collectionTree AS
WITH RECURSIVE colTree AS ( -- overlays all libraries
    SELECT
        collectionID,
        collectionName,
        collectionName path,
        0 as generation
    FROM collections cl
    WHERE cl.parentCollectionID IS NULL

    UNION

    SELECT
        ch.collectionID,
        ch.collectionName,
        p.path || '/' || ch.collectionName,
        p.generation + 1
    FROM collections ch, colTree p
    WHERE ch.parentCollectionID=p.collectionID
)
SELECT * FROM colTree;

CREATE TEMPORARY VIEW attachments AS
WITH titles AS (
    SELECT
        itemData.itemID itemID,
        itemDataValues.value title

    FROM itemData
    JOIN itemDataValues ON itemData.valueID=itemDataValues.valueID
    JOIN fields ON fields.fieldID=itemData.fieldID

    WHERE fields.fieldName='title'
),
authorData AS (
    SELECT
        agg.itemID itemID,
        agg.cnt authorsNumber,
        fl.creatorID firstAuthorID

    FROM (
        SELECT
            cr.itemID itemID,
            COUNT(*) cnt,
            MIN(cr.orderIndex) minOrderIndex

            FROM itemCreators cr
            JOIN creatorTypes tp ON tp.creatorTypeID=cr.creatorTypeID

            WHERE tp.creatorType='author'
            GROUP BY cr.itemID
    ) agg
    JOIN itemCreators fl ON fl.itemID=agg.itemID AND fl.orderIndex=agg.minOrderIndex
),
atts AS (
    SELECT 
        i.key key, 
        titles.title title, 
        creators.firstName authorFirstName,
        creators.lastName authorLastName,
        authorData.authorsNumber numberOfAuthors,
        itemAttachments.path storagePath,
        collectionTree.path collectionPath

    FROM itemAttachments
    JOIN items p ON p.itemID=COALESCE(itemAttachments.parentItemID, itemAttachments.itemID)
    JOIN items i ON i.itemID=itemAttachments.itemID

    JOIN titles ON titles.itemID=p.itemID

    LEFT JOIN authorData ON authorData.itemID=p.itemID
    LEFT JOIN creators ON creators.creatorID=authorData.firstAuthorID

    JOIN collectionItems ON p.itemID=collectionItems.itemID
    JOIN collectionTree ON collectionItems.collectionID=collectionTree.collectionID

    WHERE itemAttachments.path IS NOT NULL
)
SELECT * FROM atts;
