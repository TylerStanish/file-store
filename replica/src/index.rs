use std::fmt::Debug;
use std::fs::{self, File};
use std::hash::Hasher;
use std::io::{BufRead, BufReader};
use std::path::Path;
use inotify::{
    Inotify,
    WatchMask,
};
use twox_hash::XxHash64;
use fxhash;
use crc32fast;

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
/// If the path points to a directory, it will
/// hash all files in that directory
pub fn hash<P: AsRef<Path> + Debug>(path: &P) {
    let mut hasher = XxHash64::default();
    //let mut hasher = fxhash::FxHasher64::default();
    //let mut hasher = crc32fast::Hasher::default();
    let file = File::open(path).expect("Invalid file path");
    if file.metadata().unwrap().is_dir() {
        for entry_result in fs::read_dir(path).unwrap() {
            let entry = entry_result.unwrap();
            hash(&entry.path())
        }
        return;
    }
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
    let hash = hasher.finish();
    println!("{:?} hashed to {:#x?}", path, hash);
}
