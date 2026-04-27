use rayon::prelude::*;
use std::path::PathBuf;
use std::fs;

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
                fs::metadata(path).map(|m| m.len()).unwrap_or(0)
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

fn get_size(path: &PathBuf) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}