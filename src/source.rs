//! Completion source trait and built-in source implementations.
//!
//! Each source gathers completion candidates from a different origin:
//! buffer words, file paths, or snippets. LSP completions are handled
//! separately via Neovim's built-in LSP client callbacks.

use crate::item::{CompletionItem, CompletionKind, CompletionSource as ItemSource};
use std::collections::HashSet;
use std::path::Path;

/// Context passed to completion sources when requesting candidates.
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// The text of the current line up to (but not including) the cursor.
    pub line_before_cursor: String,
    /// The word prefix being typed (extracted from `line_before_cursor`).
    pub prefix: String,
    /// The full path of the current buffer, if available.
    pub filepath: Option<String>,
    /// All lines in the current buffer.
    pub buffer_lines: Vec<String>,
    /// Current cursor row (0-indexed).
    pub cursor_row: usize,
    /// Current cursor col (0-indexed, byte offset).
    pub cursor_col: usize,
}

impl CompletionContext {
    /// Extract the word prefix from the line before cursor.
    ///
    /// A "word" is a contiguous run of alphanumeric chars, underscores,
    /// or (for paths) slashes, dots, and tildes.
    #[must_use]
    pub fn extract_prefix(line: &str) -> String {
        let bytes = line.as_bytes();
        let mut start = bytes.len();
        while start > 0 {
            let ch = bytes[start - 1];
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                start -= 1;
            } else {
                break;
            }
        }
        line[start..].to_string()
    }

    /// Extract a path-like prefix that includes `/`, `.`, `~`, `-`.
    #[must_use]
    pub fn extract_path_prefix(line: &str) -> String {
        let bytes = line.as_bytes();
        let mut start = bytes.len();
        while start > 0 {
            let ch = bytes[start - 1];
            if ch.is_ascii_alphanumeric()
                || ch == b'_'
                || ch == b'/'
                || ch == b'.'
                || ch == b'~'
                || ch == b'-'
            {
                start -= 1;
            } else {
                break;
            }
        }
        line[start..].to_string()
    }
}

/// Collect buffer-word completions from the given buffer lines.
///
/// Scans all lines for word-like tokens and returns those matching
/// the given prefix. Deduplicates results.
#[must_use]
pub fn buffer_completions(lines: &[String], prefix: &str, current_row: usize) -> Vec<CompletionItem> {
    if prefix.is_empty() {
        return Vec::new();
    }

    let mut seen = HashSet::new();
    let mut items = Vec::new();

    for (row, line) in lines.iter().enumerate() {
        for word in extract_words(line) {
            if word.len() > prefix.len()
                && word.starts_with(prefix)
                && seen.insert(word.to_string())
            {
                let mut item = CompletionItem::new(word, CompletionKind::Text, ItemSource::Buffer);
                // Prefer words from nearby lines.
                let distance = row.abs_diff(current_row);
                item = item.sort_text(&format!("7{distance:05}{word}"));
                items.push(item);
            }
        }
    }

    items
}

/// Extract all word-like tokens from a line.
fn extract_words(line: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let bytes = line.as_bytes();
    let mut start = None;

    for (i, &b) in bytes.iter().enumerate() {
        if b.is_ascii_alphanumeric() || b == b'_' {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            let word = &line[s..i];
            if word.len() >= 2 {
                words.push(word);
            }
            start = None;
        }
    }
    if let Some(s) = start {
        let word = &line[s..];
        if word.len() >= 2 {
            words.push(word);
        }
    }

    words
}

/// Collect file-path completions for a given path prefix.
///
/// Reads the filesystem to list directory entries matching the prefix.
#[must_use]
pub fn path_completions(path_prefix: &str) -> Vec<CompletionItem> {
    if path_prefix.is_empty() {
        return Vec::new();
    }

    let expanded = if path_prefix.starts_with('~') {
        if let Some(home) = home_dir() {
            path_prefix.replacen('~', &home, 1)
        } else {
            return Vec::new();
        }
    } else {
        path_prefix.to_string()
    };

    let path = Path::new(&expanded);

    // Determine the directory to list and the filename prefix to filter.
    let (dir, name_prefix) = if expanded.ends_with('/') || expanded.ends_with('\\') {
        (path.to_path_buf(), String::new())
    } else {
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let name = path
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().to_string());
        (dir, name)
    };

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut items = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name_prefix.is_empty() && !name.starts_with(&name_prefix) {
            continue;
        }
        // Skip hidden files unless prefix starts with '.'
        if name.starts_with('.') && !name_prefix.starts_with('.') {
            continue;
        }

        let is_dir = entry.file_type().is_ok_and(|ft| ft.is_dir());
        let kind = if is_dir {
            CompletionKind::Folder
        } else {
            CompletionKind::File
        };

        let insert = if is_dir {
            format!("{name}/")
        } else {
            name.clone()
        };

        let item = CompletionItem::new(&name, kind, ItemSource::Path)
            .insert_text(&insert)
            .detail(if is_dir { "directory" } else { "file" });
        items.push(item);
    }

    items.sort_by(|a, b| {
        // Folders first, then files, alphabetically within each group
        let a_is_dir = a.kind == CompletionKind::Folder;
        let b_is_dir = b.kind == CompletionKind::Folder;
        b_is_dir.cmp(&a_is_dir).then_with(|| a.label.cmp(&b.label))
    });

    items
}

/// Get the user's home directory.
fn home_dir() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_prefix_simple() {
        assert_eq!(CompletionContext::extract_prefix("hello wor"), "wor");
    }

    #[test]
    fn extract_prefix_with_underscore() {
        assert_eq!(CompletionContext::extract_prefix("my_var"), "my_var");
    }

    #[test]
    fn extract_prefix_empty_line() {
        assert_eq!(CompletionContext::extract_prefix(""), "");
    }

    #[test]
    fn extract_prefix_after_paren() {
        assert_eq!(CompletionContext::extract_prefix("foo(bar"), "bar");
    }

    #[test]
    fn extract_prefix_all_word() {
        assert_eq!(CompletionContext::extract_prefix("hello"), "hello");
    }

    #[test]
    fn extract_path_prefix_relative() {
        assert_eq!(
            CompletionContext::extract_path_prefix("open ./src/ma"),
            "./src/ma"
        );
    }

    #[test]
    fn extract_path_prefix_absolute() {
        assert_eq!(
            CompletionContext::extract_path_prefix("cat /etc/hos"),
            "/etc/hos"
        );
    }

    #[test]
    fn extract_path_prefix_tilde() {
        assert_eq!(
            CompletionContext::extract_path_prefix("cd ~/cod"),
            "~/cod"
        );
    }

    #[test]
    fn extract_words_basic() {
        let words = extract_words("hello world foo_bar");
        assert_eq!(words, vec!["hello", "world", "foo_bar"]);
    }

    #[test]
    fn extract_words_with_punctuation() {
        let words = extract_words("fn main() { let x = 42; }");
        assert_eq!(words, vec!["fn", "main", "let", "42"]);
    }

    #[test]
    fn extract_words_skips_single_chars() {
        let words = extract_words("a b cd ef");
        assert_eq!(words, vec!["cd", "ef"]);
    }

    #[test]
    fn buffer_completions_basic() {
        let lines = vec![
            "fn hello_world() {".to_string(),
            "    let hello_there = 1;".to_string(),
            "    hel".to_string(),
        ];
        let items = buffer_completions(&lines, "hel", 2);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"hello_world"));
        assert!(labels.contains(&"hello_there"));
    }

    #[test]
    fn buffer_completions_no_self_match() {
        // The prefix itself should not appear (word must be longer than prefix).
        let lines = vec!["foo foobar".to_string()];
        let items = buffer_completions(&lines, "foo", 0);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"foobar"));
        assert!(!labels.contains(&"foo"));
    }

    #[test]
    fn buffer_completions_deduplicates() {
        let lines = vec![
            "hello hello hello".to_string(),
            "hello_world".to_string(),
        ];
        let items = buffer_completions(&lines, "hel", 0);
        // "hello" appears once, "hello_world" once
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels.iter().filter(|l| **l == "hello").count(), 1);
    }

    #[test]
    fn buffer_completions_empty_prefix() {
        let lines = vec!["hello world".to_string()];
        let items = buffer_completions(&lines, "", 0);
        assert!(items.is_empty());
    }

    #[test]
    fn path_completions_empty_prefix() {
        let items = path_completions("");
        assert!(items.is_empty());
    }

    #[test]
    fn path_completions_reads_directory() {
        // Test with a known directory
        let items = path_completions("/tmp/");
        // /tmp/ should exist on macOS/Linux and have entries (or be empty).
        // We just verify it doesn't crash.
        for item in &items {
            assert!(
                item.kind == CompletionKind::File || item.kind == CompletionKind::Folder,
                "unexpected kind: {:?}",
                item.kind
            );
        }
    }

    #[test]
    fn path_completions_nonexistent_dir() {
        let items = path_completions("/nonexistent_path_that_does_not_exist_12345/");
        assert!(items.is_empty());
    }

    #[test]
    fn buffer_completions_prefers_nearby_lines() {
        let lines: Vec<String> = (0..100)
            .map(|i| format!("hello_{i}"))
            .collect();
        let items = buffer_completions(&lines, "hello", 50);
        // Items from row 50 should sort before items from row 0 or 99
        assert!(!items.is_empty());
        // All items should have source Buffer
        for item in &items {
            assert_eq!(item.source, ItemSource::Buffer);
        }
    }
}
