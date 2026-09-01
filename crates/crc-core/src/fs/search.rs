use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::{BinaryDetection, SearcherBuilder};
use ignore::{WalkBuilder, WalkState};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextQuery {
    pub pattern: String,
    #[serde(default)]
    pub is_regex: bool,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    #[serde(default = "default_max_per_file")]
    pub max_matches_per_file: usize,
}

fn default_max_files() -> usize {
    500
}

fn default_max_per_file() -> usize {
    50
}

impl TextQuery {
    pub fn literal(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            is_regex: false,
            case_sensitive: false,
            max_files: default_max_files(),
            max_matches_per_file: default_max_per_file(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMatches {
    pub path: PathBuf,
    pub matches: Vec<LineMatch>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineMatch {
    pub line: u64,
    pub text: String,
}

pub fn search_text(root: &Path, query: &TextQuery) -> Result<Vec<FileMatches>> {
    let pattern = if query.is_regex {
        query.pattern.clone()
    } else {
        regex_escape(&query.pattern)
    };

    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(!query.case_sensitive)
        .build(&pattern)
        .map_err(|e| CoreError::Pattern(Box::new(e)))?;

    let results = Mutex::new(Vec::new());
    let files_hit = AtomicUsize::new(0);
    let max_per_file = query.max_matches_per_file;
    let max_files = query.max_files;

    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .filter_entry(|entry| !is_noise(entry.path()))
        .build_parallel()
        .run(|| {
            let matcher = matcher.clone();
            let results = &results;
            let files_hit = &files_hit;
            let mut searcher = SearcherBuilder::new()
                .binary_detection(BinaryDetection::quit(0))
                .line_number(true)
                .build();

            Box::new(move |entry| {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => return WalkState::Continue,
                };
                if !entry.file_type().is_some_and(|t| t.is_file()) {
                    return WalkState::Continue;
                }
                if files_hit.load(Ordering::Relaxed) >= max_files {
                    return WalkState::Quit;
                }

                let mut found = Vec::new();
                let sink = UTF8(|line, text| {
                    found.push(LineMatch {
                        line,
                        text: text.trim_end().to_string(),
                    });
                    Ok(found.len() < max_per_file)
                });
                if searcher.search_path(&matcher, entry.path(), sink).is_err() {
                    return WalkState::Continue;
                }

                if !found.is_empty() {
                    let path = entry.path();
                    let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
                    results.lock().unwrap().push(FileMatches {
                        path: relative,
                        matches: found,
                    });
                    if files_hit.fetch_add(1, Ordering::Relaxed) + 1 >= max_files {
                        return WalkState::Quit;
                    }
                }
                WalkState::Continue
            })
        });

    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

pub fn find_files(root: &Path, query: &str, limit: usize) -> Vec<PathBuf> {
    let needle = query.to_lowercase();
    let mut scored: Vec<(i32, PathBuf)> = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .filter_entry(|entry| !is_noise(entry.path()))
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(path);
        let haystack = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
            .collect::<Vec<_>>()
            .join("/");
        if let Some(score) = fuzzy_score(&haystack, &needle) {
            scored.push((score, relative.to_path_buf()));
        }
    }

    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.as_os_str().len().cmp(&b.1.as_os_str().len()))
    });
    scored.into_iter().take(limit).map(|(_, p)| p).collect()
}

fn fuzzy_score(haystack: &str, needle: &str) -> Option<i32> {
    if needle.is_empty() {
        return Some(0);
    }
    let basename_start = haystack.rfind('/').map_or(0, |i| i + 1);

    let mut score = 0;
    let mut last_index = None;
    let mut chars = needle.chars();
    let mut wanted = chars.next()?;

    for (index, ch) in haystack.char_indices() {
        if ch != wanted {
            continue;
        }
        score += 1;
        if last_index == Some(index.saturating_sub(1)) {
            score += 4;
        }
        if index >= basename_start {
            score += 2;
        }
        last_index = Some(index);
        match chars.next() {
            Some(next) => wanted = next,
            None => return Some(score),
        }
    }
    None
}

fn is_noise(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|n| n.to_str()),
        Some(".git" | "target" | "node_modules" | "dist" | ".next")
    )
}

fn regex_escape(literal: &str) -> String {
    const SPECIAL: &str = r"\.+*?()|[]{}^$#&-~";
    let mut out = String::with_capacity(literal.len());
    for ch in literal.chars() {
        if SPECIAL.contains(ch) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}
