use std::collections::HashMap;
use std::io::Read;
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
use serde::{Deserialize, Serialize};
use serde_json::json;
use twox_hash::XxHash64;
use fxhash;
use crc32fast;


#[derive(Serialize, Deserialize, Clone)]
pub struct IndexItem {
    file_path: String,
    file_size: u64,
    hash: u64,
    hash_stopped_at: u64,
    last_modified: SystemTime,
}

impl IndexItem {
    pub fn new(file_path: String) -> Self {
        let file = File::open(&file_path).expect("Invalid file path");
        let metadata = file.metadata().unwrap();
        assert_eq!(metadata.is_file(), true);
        IndexItem {
            file_path,
            file_size: metadata.len(),
            hash: 0,
            hash_stopped_at: 0,
            last_modified: metadata.modified().expect("Your OS does not support 'last modified' metadata"),
        }
    }
}

/// Data structure representing the local file index
/// on this node.
/// 
/// Do we also want to keep the items together by `file_size` sorted by
/// another field like `hash_stopped_at`?
#[derive(Serialize, Deserialize)]
pub struct LocalIndex {
    entries: HashMap<u64, Vec<IndexItem>>,
}

impl LocalIndex {
    pub fn new() -> Self {
        LocalIndex {
            entries: HashMap::new(),
        }
    }

    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).expect("Corrupted or invalid index contents")
    }

    pub fn index(&mut self, path: &str) {
        let file = File::open(path).expect("Invalid file path");
        if file.metadata().unwrap().is_dir() {
            for entry_result in fs::read_dir(path).unwrap() {
                let entry = entry_result.unwrap();
                self.index(&entry.path().to_string_lossy().to_string())
            }
        } else {
            self.put(&IndexItem::new(path.to_string()))
        }
    }

    fn put(&mut self, index_item: &IndexItem) {
        let size = index_item.file_size;
        let contains = self.entries.contains_key(&size);
        if !contains {
            let mut v = Vec::new();
            v.push(index_item.clone());
            self.entries.insert(index_item.file_size, v);
            return;
        }
        self.entries.get_mut(&size).unwrap().push(index_item.clone());
        self.hash_all(size);
    }

    fn hash_all(&mut self, size: u64) {
        for item in self.entries.get_mut(&size).unwrap() {
            if item.hash_stopped_at == item.file_size { continue; }
            item.hash = hash_file(&item.file_path);
        }
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
