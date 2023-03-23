# Zotero FUSE

View your zotero library as a local directory structure.
Currently, it is implemented only using symlinks instead of true FUSE (userspace filesystem).

This program
1. reads the local sqlite zotero database.
2. creates a directory tree representing the collections and subcollections
3. creates symlinks for all items with attachments (the symlink names being "1st author (et. al.) - title")

For now, paths to database and fuse are set using variables inside zotefuse.py, so change them accordingly.
