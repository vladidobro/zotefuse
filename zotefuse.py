#!/usr/bin/env python

import os
import sqlite3

PREFIX_ZOTERO = '/data/references/'
PREFIX_ZOTERO_STORAGE = PREFIX_ZOTERO + 'storage/'
ZOTERO_DATABASE = PREFIX_ZOTERO + 'zotero.sqlite'
PREFIX_FUSE = PREFIX_ZOTERO + 'tree/'
SQL_FILE = 'views.sql'


def sanitize_filename(path):
    '''Remove unwanted characters.'''
    forbidden = '\\/:*?"<>|'
    for c in forbidden:
        path = path.replace(c, '')
    return path


def judge_unique():
    '''Send path and receive if valid.'''
    values = []
    accept = None
    while True:
        candidate = yield accept
        if candidate in values or os.path.exists(candidate):
            accept = False
        else:
            values += [candidate]
            accept = True


def path_generator(orig_path):
    '''Generate paths for duplicate files using ~n~.'''
    def path_from_n(n):
        return root + ('~{}~'.format(n) if n else '') + ext
    root, ext = os.path.splitext(orig_path)
    n = 0
    while True:
        yield path_from_n(n)
        n += 1


def path_validator():
    '''Send candidate path and receive the one you should use.'''
    judge = judge_unique()
    next(judge)
    candidate_path = None
    while True:
        orig_path = yield candidate_path
        path_gen = path_generator(orig_path)
        candidate_path = orig_path
        while not judge.send(candidate_path):
            candidate_path = next(path_gen)


class Attachment():
    '''Class for attachment instances. Computes paths etc.'''
    def __init__(self, prop_dict):
        self.key = prop_dict['key']
        self.title = prop_dict['title']
        self.author_first = prop_dict['authorFirstName']
        self.author_last = prop_dict['authorLastName']
        self.author_number = prop_dict['numberOfAuthors'] or 0
        self.storage_path = prop_dict['storagePath']
        self.collection_path = prop_dict['collectionPath']
        self.is_local = self.storage_path.startswith('storage:')
        _, self.extension = os.path.splitext(self.src_filename)

    @property
    def src_filename(self):
        if self.is_local:
            return self.storage_path[len('storage:'):]
        else:
            _, filename = os.path.split(self.storage_path)
            return filename

    @property
    def src_path(self):
        if self.is_local:
            return PREFIX_ZOTERO_STORAGE + self.key + \
                    '/' + self.src_filename
        else:
            return self.storage_path

    @property
    def dst_ideal_fnroot(self):
        author_prefix = '' if self.author_number == 0 else \
                self.author_last + \
                ('' if self.author_number == 1 else ' et al.') + \
                ' - '
        return sanitize_filename(author_prefix + self.title)

    @property
    def dst_ideal_filename(self):
        if self.author_number == 0 and self.title == self.src_filename:
            return self.src_filename
        else:
            return self.dst_ideal_fnroot + self.extension

    @property
    def dst_ideal_path(self):  # fnroot = filename root
        return PREFIX_FUSE + self.collection_path + '/' + \
                self.dst_ideal_filename


def query_db():
    '''Download relevant data from zotero database.'''
    con = sqlite3.connect(ZOTERO_DATABASE)
    con.row_factory = sqlite3.Row
    cur = con.cursor()

    with open(SQL_FILE, 'r') as sql_file:
        sql_views = sql_file.read()
    cur.executescript(sql_views)
    tree = cur.execute('SELECT * FROM collectionTree;').fetchall()
    items = cur.execute('SELECT * FROM attachments;').fetchall()
    con.close()

    items = list(map(Attachment, items))
    return tree, items


def clean_tree(tree):
    '''Delete all symlinks and then empty directories not in tree.'''
    os.makedirs(PREFIX_FUSE, exist_ok=True)
    file_gen = os.walk(PREFIX_FUSE)
    for root, dirs, files in file_gen:
        for f in files:
            path = os.path.join(root, f)
            if os.path.islink(path):
                os.remove(path)

    def delete_if_should(path):
        with os.scandir(path) as entries:
            mark_for_deletion = True
            for d in filter(lambda d: d.is_dir(follow_symlinks=False),
                            entries):
                if delete_if_should(d.path):
                    try:
                        os.rmdir(d.path)
                        print(d.path)
                    except OSError:
                        pass
                else:
                    mark_for_deletion = False
            if path in map(lambda r: PREFIX_FUSE+r['path'], tree):
                return False
            if any(filter(lambda f: f.is_file(follow_symlinks=False),
                          entries)):
                return False
            return mark_for_deletion
    delete_if_should(PREFIX_FUSE)


def make_tree(tree):
    '''Create directory structure mimicking zotero collections.'''
    for row in tree:
        path = PREFIX_FUSE+row['path']
        os.makedirs(path, exist_ok=True)


def generate_dst_paths(items):
    '''Generate link destination paths for all attachments.
    Reads preferred (item.dst_ideal_path) path and modifies if exists.
    Identical file names are appended with ~n~.'''
    validator = path_validator()
    next(validator)
    for item in items:
        item.dst_path = validator.send(item.dst_ideal_path)


def make_links(items):
    '''Create symlink structure.'''
    for item in items:
        os.symlink(item.src_path, item.dst_path)


def main():
    tree, items = query_db()
    clean_tree(tree)
    make_tree(tree)
    generate_dst_paths(items)
    make_links(items)


if __name__ == '__main__':
    main()
