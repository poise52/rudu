use crossbeam_channel::Sender;
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

/// One pass over all files under `root`: each directory path maps to total allocated size
/// (blocks × 512) of all files in its subtree, matching prior per-file semantics.
pub fn build_size_index(root: PathBuf) -> HashMap<PathBuf, u64> {
    let root = fs::canonicalize(&root).unwrap_or(root);
    let mut map = HashMap::new();

    let walker = walkdir::WalkDir::new(&root)
        .same_file_system(true)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let size = meta.blocks() * 512;
        add_size_to_ancestors(&mut map, root.as_path(), path, size);
    }

    map
}

fn add_size_to_ancestors(
    map: &mut HashMap<PathBuf, u64>,
    root: &Path,
    file_path: &Path,
    size: u64,
) {
    let Some(mut dir) = file_path.parent().map(Path::to_path_buf) else {
        return;
    };
    loop {
        if !dir.starts_with(root) {
            break;
        }
        *map.entry(dir.clone()).or_insert(0) += size;
        if dir.as_path() == root {
            break;
        }
        let Some(parent) = dir.parent() else {
            break;
        };
        dir = parent.to_path_buf();
    }
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
            let meta = fs::metadata(&path).ok()?;
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
