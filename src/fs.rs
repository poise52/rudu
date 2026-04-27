use rayon::prelude::*;
use std::path::PathBuf;
use std::fs;
use std::os::unix::fs::MetadataExt;
use crossbeam_channel::Sender;

pub struct DirEntry {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

pub fn scan_dir(path: &str) -> Vec<DirEntry> {
    let read = match fs::read_dir(path) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let paths: Vec<PathBuf> = read
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    let mut entries: Vec<DirEntry> = paths
        .par_iter()
        .map(|path| {
            let is_dir = path.is_dir();
            let size = if is_dir {
                get_size(path)
            } else {
                fs::metadata(path)
                    .map(|m| m.blocks() * 512)
                    .unwrap_or(0)
            };
            DirEntry {
                path: path.clone(),
                size,
                is_dir,
            }
        })
        .collect();

    entries.sort_by(|a, b| b.size.cmp(&a.size));
    entries
}

pub fn scan_dir_async(path: PathBuf, tx: Sender<Vec<DirEntry>>) {
    std::thread::spawn(move || {
        let entries = scan_dir(path.to_str().unwrap_or(""));
        let _ = tx.send(entries);
    });
}

fn get_size(path: &PathBuf) -> u64 {
    walkdir::WalkDir::new(path)
        .same_file_system(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.blocks() * 512)
        .sum()
}