use crossbeam_channel::Sender;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type SizeIndex = Arc<HashMap<PathBuf, u64>>;

pub struct DirEntry {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

pub fn build_size_index(root: PathBuf) -> HashMap<PathBuf, u64> {
    let root = match fs::canonicalize(&root) {
        Ok(p) => p,
        Err(_) => root,
    };
    let root_dev = fs::metadata(&root).map(|m| m.dev()).unwrap_or(0);
    let (sizes, _) = scan_tree_parallel(root.as_path(), root_dev);
    sizes
}

fn scan_tree_parallel(path: &Path, root_dev: u64) -> (HashMap<PathBuf, u64>, u64) {
    let read = match fs::read_dir(path) {
        Ok(r) => r,
        Err(_) => {
            let mut m = HashMap::new();
            m.insert(path.to_path_buf(), 0);
            return (m, 0);
        }
    };

    let mut files_total = 0u64;
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut cross_fs: Vec<PathBuf> = Vec::new();

    for entry in read.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let p = entry.path();
        if meta.is_file() {
            files_total += meta.blocks() * 512;
        } else if meta.is_dir() {
            if meta.dev() == root_dev {
                dirs.push(p);
            } else {
                cross_fs.push(p);
            }
        }
    }

    let mut sizes = HashMap::new();
    let mut total = files_total;

    if dirs.len() > 1 {
        let parts: Vec<(HashMap<PathBuf, u64>, u64)> = dirs
            .par_iter()
            .map(|d| scan_tree_parallel(d.as_path(), root_dev))
            .collect();
        for (sub, t) in parts {
            sizes.extend(sub);
            total += t;
        }
    } else {
        for d in dirs {
            let (sub, t) = scan_tree_parallel(&d, root_dev);
            sizes.extend(sub);
            total += t;
        }
    }

    for p in cross_fs {
        sizes.insert(p, 0);
    }
    sizes.insert(path.to_path_buf(), total);
    (sizes, total)
}

pub fn list_dir(sizes: &HashMap<PathBuf, u64>, dir: &Path) -> Vec<DirEntry> {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let mut entries: Vec<DirEntry> = read
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            let meta = e.metadata().ok()?;
            let is_dir = meta.is_dir();
            let size = if is_dir {
                sizes.get(&path).copied().unwrap_or(0)
            } else {
                meta.blocks() * 512
            };
            Some(DirEntry {
                path,
                size,
                is_dir,
            })
        })
        .collect();

    entries.sort_by(|a, b| b.size.cmp(&a.size));
    entries
}

pub fn build_index_async(root: PathBuf, tx: Sender<SizeIndex>) {
    std::thread::spawn(move || {
        let map = build_size_index(root);
        let _ = tx.send(Arc::new(map));
    });
}
