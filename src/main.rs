use std::fmt::Debug;
use std::fs::{self, File};
use std::hash::Hasher;
use std::path::Path;
use std::io::{BufRead, BufReader};
use twox_hash::XxHash64;
use fxhash;
use crc32fast;

/// Will hash a file
/// If the path points to a directory, it will
/// hash all files in that directory
fn hash<P: AsRef<Path> + Debug>(path: &P) {
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


fn main() {
    hash(&Path::new("."));
}
