use std::fmt::Debug;
use std::fs::{self, File};
use std::hash::Hasher;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;
use inotify::{
    Inotify,
    WatchMask,
};
use twox_hash::XxHash64;
use fxhash;
use crc32fast;


#[derive(Clone)]
pub struct IndexItem {
    file_path: String,
    file_size: u32,
    hash: u64,
    hash_stopped_at: u32,
    last_modified: SystemTime,
}

/// Data structure representing the local file index
/// on this node. The LocalIndex will always be sorted by
/// `file_size` so we can perform O(log(n)) lookups for
/// the most common type of indexing collision: files that
/// have the same size.
/// 
/// Do we also want to keep the items together by `file_size` sorted by
/// another field like `hash_stopped_at`?
pub struct LocalIndex {
    entries: Vec<IndexItem>,
}

impl LocalIndex {
    pub fn new() -> Self {
        LocalIndex {
            entries: Vec::new(),
        }
    }

    pub fn put(&mut self, index_item: &IndexItem) {
        match self.entries.binary_search_by_key(&index_item.file_size, |item| item.file_size) {
            Ok(other_index) => {
                let other_entry = self.entries.get(other_index).unwrap().to_owned();
                self.resolve_size_collision(index_item, &other_entry);
                self.entries.insert(other_index, index_item.clone());
            }
            Err(index) => self.entries.insert(index, index_item.clone()),
        }
    }

    fn resolve_size_collision(&mut self, item: &IndexItem, other: &IndexItem) {
        assert_eq!(item.file_size, other.file_size);
        // for now, just hash the entire file
        if other.hash_stopped_at < other.file_size {
            other.hash = hash_file(&other.file_path)
        }
        item.hash = hash_file(&item.file_path)
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
