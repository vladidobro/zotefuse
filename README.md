ZoteroFS
========

Mount your Zotero library as a FUSE filesystem.
All attachments are symlinked in a diretory tree mirroring the Zotero collections.

Installation
------------
```
$ cargo install zoterofs
```

Usage
-----
```
$ export ZOTEROHOME=/home/me/zotero
$ export MOUNTPOINT=$ZOTEROHOME/mnt
$ zoterofs $ZOTEROHOME $MOUNTPOINT

$ ls $MOUNTPOINT
MyCollection
```

TODO
----

- Fstab example
- Hot reloading
