use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions, canonicalize};
use std::hash::Hasher;
use std::io::{BufRead, BufReader, Read, Write, ErrorKind};
use std::path::Path;
use std::time::SystemTime;
use inotify::{
    Inotify,
    WatchMask,
};
use serde::{Deserialize, Serialize};
use serde_json;
use twox_hash::XxHash64;
use fxhash;
use crc32fast;

/// A tag is a logical grouping of files (although it doesn't have to be).
/// Furthermore, tags are the parent directory by which any sub files will be resolved.
/// We have this so you can easily move/rename indexed files/subdirectories
/// and so you don't have to deal with absolute file paths specific to a single computer/node
/// from the dashboard/management console.
/// These can be separate drives or root folders on which you would like to index
/// 
/// Do we also want to keep the items together by `file_size` sorted by
/// another field like `hash_stopped_at`?
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    pub abs_path: String,
    pub name: String,
    pub entries: HashMap<u64, Vec<IndexItem>>,
    pub paths: HashMap<String, IndexItem>, // stores each file by path to look up/get file info in O(1). Stores abs file path
}

impl Tag {
    pub fn new(name: &str, abs_path: &str) -> Self {
        Tag {
            abs_path: abs_path.to_owned(),
            name: name.to_owned(),
            entries: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// Gets redundancies within a tag
    pub fn redundancies(&self) -> Vec<Vec<IndexItem>> {
        let mut res = Vec::new();
        for (key, val) in &self.entries {
            let mut set = HashSet::new();
            let mut dups = Vec::new();
            for index_item in val {
                let tuple = (index_item.hash, index_item.hash_stopped_at);
                if set.contains(&tuple) {
                    dups.push(index_item.clone());
                } else {
                    set.insert(tuple.clone());
                }
            }
            if !dups.is_empty() {
                res.push(dups);
            }
        }
        res
    }

    /// Index a tag at the top level recursively, so abs_path MUST point to a directory
    pub fn index(&mut self) {
        let full_abs_path = self.abs_path.to_owned();
        let file = File::open(&full_abs_path).unwrap();
        assert!(file.metadata().unwrap().is_dir());
        // first take care of the deleted items by traversing the index
        // here you can also take care of the modified files and the files that stay the same
        for (path, index_item) in self.paths.clone() { // we want to clone so we can modify this while in the for loop
            match index_item.file_changed_event() {
                FileChangeEvent::Deleted => {
                    // remove from the index (remember self.entries and self.paths)
                    let item = self.paths.remove(&path).unwrap();
                    let pos = self.entries[&item.file_size].iter().position(|ii| ii.file_path == item.file_path).unwrap();
                    self.entries.get_mut(&item.file_size).unwrap().remove(pos);
                },
                FileChangeEvent::Modified => {
                    // check the size of the file, and remove from self.entries[size] and insert into new key's vec (and hash) if necessary
                    let new_size = File::open(path).unwrap().metadata().unwrap().len();
                    if index_item.file_size != new_size {
                        let pos = self.entries[&index_item.file_size].iter().position(|ii| ii.file_path == index_item.file_path).unwrap();
                        self.entries.get_mut(&index_item.file_size).unwrap().remove(pos);
                        self.put(&IndexItem::new(index_item.file_path, self));
                    } else {
                        // even though the size of the file is the same, it doesn't mean it wasn't modified. Re-hash anyways
                        self.hash_all(new_size);
                    }
                },
                FileChangeEvent::NoChange => (),
                FileChangeEvent::Created => panic!("A file was claimed to be created when it was already in the index"),
            }
        }
        // now you must find the newly created files by traversing the file system
        for entry_result in fs::read_dir(&full_abs_path).unwrap() {
            let entry = entry_result.unwrap();
            let path_string = entry.path().to_string_lossy().to_string();
            if self.paths.contains_key(&path_string) {
                continue;
            }
            self.index_abs(&path_string);
        }
    }

    fn index_abs(&mut self, full_abs_path: &str) {
        println!("Indexing: {}", full_abs_path);
        let file = File::open(&full_abs_path).expect("Invalid file path");
        if file.metadata().unwrap().is_dir() {
            for entry_result in fs::read_dir(&full_abs_path).unwrap() {
                let entry = entry_result.unwrap();
                self.index_abs(&entry.path().to_string_lossy().to_string());
            }
        } else {
            self.put(&IndexItem::new(full_abs_path.to_owned(), self));
        }
    }

    /// Assumes the `index_item` is not already in the index
    fn put(&mut self, index_item: &IndexItem) {
        let size = index_item.file_size;
        let contains = self.entries.contains_key(&size);
        if !contains {
            let mut v = Vec::new();
            v.push(index_item.clone());
            self.entries.insert(index_item.file_size, v);
            self.paths.insert(index_item.absolute_path(), index_item.clone());
            return;
        }
        self.entries.get_mut(&size).unwrap().push(index_item.clone());
        self.paths.insert(index_item.absolute_path(), index_item.clone());
        self.hash_all(size);
    }

    fn hash_all(&mut self, size: u64) {
        for item in self.entries.get_mut(&size).unwrap() {
            if item.hash_stopped_at == item.file_size && !Self::modified_since_last_index(item) { continue; }
            item.hash = hash_file(&item.file_path);
            item.hash_stopped_at = item.file_size;
        }
    }

    fn modified_since_last_index(item: &IndexItem) -> bool {
        return File::open(&item.file_path).unwrap().metadata().unwrap().modified().unwrap() <= item.last_modified;
    }
}

pub enum FileChangeEvent {
    Modified,
    Deleted,
    Created,
    NoChange,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexItem {
    /// The file path relative to the path of the tag that contains this file
    pub file_path: String,
    pub file_size: u64,
    pub hash: u64,
    pub hash_stopped_at: u64,
    pub last_modified: SystemTime,
    pub tag: Tag,
    //pub tag_name: String,
}

impl IndexItem {
    pub fn new(file_path: String, tag: &Tag) -> Self {
        let file = File::open(&file_path).expect("Invalid file path");
        let metadata = file.metadata().unwrap();
        assert_eq!(metadata.is_file(), true);
        IndexItem {
            file_path,
            file_size: metadata.len(),
            hash: 0,
            hash_stopped_at: 0,
            last_modified: metadata.modified().expect("Your OS does not support 'last modified' metadata"),
            tag: tag.clone(),
        }
    }

    pub fn absolute_path(&self) -> String {
        Path::new(&self.tag.abs_path).join(&self.file_path).to_str().unwrap().to_owned()
    }

    /// The status of the file since last indexing
    pub fn file_changed_event(&self) -> FileChangeEvent {
        match File::open(self.absolute_path()) {
            Ok(file) => {
                if file.metadata().unwrap().modified().unwrap() <= self.last_modified {
                    return FileChangeEvent::Modified;
                } else {
                    return FileChangeEvent::NoChange;
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => return FileChangeEvent::Deleted,
                err => panic!("Unexpected std::io::ErrorKind {:?}", err),
            }
        }
    }
}

/// Data structure representing the local file index
/// on this node.
#[derive(Serialize, Deserialize)]
pub struct LocalIndex {
    pub tags: Vec<Tag>,
}

impl LocalIndex {
    pub fn new() -> Self {
        LocalIndex {
            tags: Vec::new(),
        }
    }

    /// Creates a new tag in this index with name `tag_name` and path `tag_path`.
    /// The path needs not be absolute
    pub fn new_tag(&mut self, tag_name: &str, tag_path: &str) {
        let canonicalized = canonicalize(tag_path).expect("Invalid path");
        let abs_tag_path = canonicalized.to_str().unwrap();
        let mut tag = Tag::new(abs_tag_path, tag_name);
        println!("Creating tag '{}' at path {}", tag_name, abs_tag_path);
        self.tags.push(tag.clone());
        tag.index(); // TODO add func like index_all or something in impl Tag{}
    }

    /// Gets redundancies across tags
    pub fn redundancies(&self) -> Vec<Vec<IndexItem>> {
        unimplemented!()
    }

    /// Finds identical files to the one found at `path`. `path` can be found
    /// anywhere, not necessarily as a child of a indexed tag
    pub fn find_matching(&self, path: &str) -> Vec<IndexItem> {
        unimplemented!()
    }

    pub fn from_local() -> Self {
        match env::home_dir() {
            Some(h) => {
                let mut home = h.to_owned();
                home.push(".config");
                home.push("file-store");
                home.push("index.json");
                if home.exists() {
                    let mut s = String::new();
                    File::open(home).unwrap().read_to_string(&mut s).unwrap();
                    return Self::from_json(&s);
                } else {
                    return Self::new();
                }
            }
            None => panic!("Your HOME is not specified, where do I get the local file index?")
        };
    }

    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).expect("Corrupted or invalid index contents")
    }

    pub fn persist_local(&self) {
        match env::home_dir() {
            Some(h) => {
                let mut home = h.to_owned();
                home.push(".config");
                home.push("file-store");
                let home_exists = home.exists();
                let mut full_file_path = home.clone();
                full_file_path.push("index.json");
                if home_exists {
                    if full_file_path.exists() {
                        OpenOptions::new().write(true).create(true).truncate(true).open(full_file_path).unwrap().write(self.to_json().as_bytes()).unwrap();
                    } else {
                        File::create(full_file_path).unwrap().write(self.to_json().as_bytes()).unwrap();
                    }
                } else {
                    fs::create_dir_all(home).unwrap();
                    File::open(full_file_path).unwrap().write(self.to_json().as_bytes()).unwrap();
                }
            }
            None => panic!("Your HOME is not specified, where do I put the local file index?")
        };
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn watch<P: AsRef<Path>>(path: &P) {
    let mut inotify = Inotify::init().expect("Could not initialize file watcher");
    inotify.add_watch(path, WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE).expect("Could not start file watcher");
    let mut buf = [0; 1024];
    let events = inotify.read_events_blocking(&mut buf).expect("Could not read events");
    for event in events {
        println!("{:?}", event);
    }
}

/// Will hash a file
pub fn hash_file<P: AsRef<Path> + Debug>(path: &P) -> u64 {
    let mut hasher = XxHash64::default();
    let file = File::open(path).expect("Invalid file path");
    assert_eq!(file.metadata().unwrap().is_file(), true);
    let mut reader = BufReader::with_capacity(1024 * 64, file);
    loop {
        let buf = reader.fill_buf().unwrap();
        let len = buf.len();
        if len == 0 {
            break;
        }
        hasher.write(buf);
        reader.consume(len);
    };
    hasher.finish()
}

#[cfg(test)]
mod test {
    use std::io::{Write, SeekFrom, Seek};
    use tempfile;
    use super::{LocalIndex, Tag};
    #[test]
    fn smoke_test_single_empty_tag() {
        let dir = tempfile::tempdir().unwrap();
        let mut index = LocalIndex::new();
        index.new_tag("vids", "./vids");
        assert_eq!(index.tags.len(), 1);
    }

    #[test]
    fn test_simple_tag_with_one_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        tag.index();
        assert_eq!(tag.paths.len(), 1);
        assert_eq!(tag.entries[&0].len(), 1);
    }

    #[test]
    fn test_simple_reindex_no_change() {
        let dir = tempfile::tempdir().unwrap();
        let file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        tag.index();
        assert_eq!(tag.paths.len(), 1);
        assert_eq!(tag.entries[&0].len(), 1);
        tag.index();
        assert_eq!(tag.paths.len(), 1);
        assert_eq!(tag.entries[&0].len(), 1);
    }

    #[test]
    fn test_simple_reindex_modified_with_same_size_but_different_content() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        file.write(&[42]).unwrap();
        tag.index();
        assert_eq!(tag.paths.len(), 1);
        assert_eq!(tag.entries[&1].len(), 1);
        file.seek(SeekFrom::Start(0)).unwrap();
        file.write(&[43]).unwrap();
        tag.index();
        assert_eq!(tag.paths.len(), 1);
        assert_eq!(tag.entries[&1].len(), 1);
    }

    #[test]
    fn test_reindex_modified_with_different_initial_size_and_different_content() {
        let dir = tempfile::tempdir().unwrap();
        let mut file1 = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let mut file2 = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        file1.write(&[42]).unwrap();
        tag.index();
        assert_eq!(tag.paths.len(), 2);
        assert_eq!(tag.entries[&0].len(), 1);
        assert_eq!(tag.entries[&1].len(), 1);
        file2.write(&[43]).unwrap();
        tag.index();
        assert_eq!(tag.paths.len(), 2);
        assert_eq!(tag.entries[&1].len(), 2);

        assert_ne!(tag.entries[&1][0].hash, tag.entries[&1][1].hash);
    }

    #[test]
    fn test_simple_reindex_modified_with_new_size() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        tag.index();
        assert_eq!(tag.paths.len(), 1);
        assert_eq!(tag.entries[&0].len(), 1);
        file.write(&[42]).unwrap();
        tag.index();
        assert_eq!(tag.paths.len(), 1);
        assert_eq!(tag.entries[&1].len(), 1);
    }

    #[test]
    fn test_files_no_collision() {
        let dir = tempfile::tempdir().unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        let mut files = Vec::new();
        for size in 0..5 {
            let mut file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
            let buf = vec![42; size];
            file.write(&buf).unwrap();
            files.push(file); // if we don't do this, drop() is called and will destroy the file, so we need to move it into the vec
        }
        tag.index();
        assert_eq!(tag.entries.len(), 5);
        assert_eq!(tag.paths.len(), 5);
        for size in 0..5 {
            assert_eq!(tag.entries.get(&size).unwrap().len(), 1);
        }
    }

    #[test]
    fn test_files_single_collision() {
        let dir = tempfile::tempdir().unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        let mut file1 = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let mut file2 = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        let buf = vec![42; 1];
        file1.write(&buf).unwrap();
        file2.write(&buf).unwrap();
        tag.index();

        assert_eq!(tag.entries.len(), 1);
        assert_eq!(tag.paths.len(), 2);
        assert_eq!(tag.entries.get(&1).unwrap().len(), 2);
        assert_eq!(tag.entries.get(&1).unwrap()[0].hash_stopped_at, 1);
    }

    #[test]
    fn test_files_many_collisions() {
        let dir = tempfile::tempdir().unwrap();
        let mut tag = Tag::new("test", dir.path().to_str().unwrap());
        let mut files = Vec::new();
        for _ in 0..5 {
            let mut file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
            let buf = vec![42; 42];
            file.write(&buf).unwrap();
            files.push(file);
        }
        tag.index();
        assert_eq!(tag.entries.len(), 1);
        assert_eq!(tag.paths.len(), 5);
        assert_eq!(tag.entries.get(&42).unwrap().len(), 5);
        for item in tag.entries.get(&42).unwrap() {
            assert_eq!(item.hash_stopped_at, 42);
        }
    }
}