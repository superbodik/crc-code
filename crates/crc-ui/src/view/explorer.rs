use std::path::{Path, PathBuf};

use crate::view::state::FileEntry;

pub fn tree(paths: &[PathBuf], limit: usize) -> Vec<FileEntry> {
    let mut sorted: Vec<&PathBuf> = paths.iter().collect();
    sorted.sort_by_cached_key(|path| order(path));

    let mut entries = Vec::new();
    let mut open: Vec<String> = Vec::new();

    for path in sorted {
        let components: Vec<String> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let Some((name, parents)) = components.split_last() else {
            continue;
        };

        let shared = parents
            .iter()
            .zip(open.iter())
            .take_while(|(a, b)| a == b)
            .count();
        open.truncate(shared);

        for (depth, folder) in parents.iter().enumerate().skip(shared) {
            if entries.len() >= limit {
                return entries;
            }
            entries.push(FileEntry::dir(folder.clone(), depth));
            open.push(folder.clone());
        }

        if entries.len() >= limit {
            return entries;
        }
        entries.push(FileEntry::file(name.clone(), parents.len()).at(path.clone()));
    }

    entries
}

fn order(path: &Path) -> Vec<(u8, String)> {
    let components: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();
    let last = components.len().saturating_sub(1);

    components
        .into_iter()
        .enumerate()
        .map(|(index, part)| (if index == last { 1 } else { 0 }, part))
        .collect()
}
