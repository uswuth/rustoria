//! std-file-processor: word-frequency counting with the standard library only.
//!
//! Reads a single text file or every `.txt` file in a directory, counts word
//! frequencies line by line with `BufRead`, and produces a small report.
//! No third-party crates are used anywhere.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Everything that can go wrong while processing input.
#[derive(Debug)]
pub enum ProcessingError {
    /// An I/O error without a specific file context.
    Io(io::Error),
    /// An I/O error tied to a specific path.
    File { path: PathBuf, source: io::Error },
    /// The input path does not exist.
    NotFound { path: PathBuf },
    /// The input directory contains no `.txt` files.
    NoTextFiles { dir: PathBuf },
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::Io(e) => write!(f, "I/O error: {e}"),
            ProcessingError::File { path, source } => {
                write!(f, "failed to process {}: {source}", path.display())
            }
            ProcessingError::NotFound { path } => {
                write!(f, "path does not exist: {}", path.display())
            }
            ProcessingError::NoTextFiles { dir } => {
                write!(f, "no .txt files found in {}", dir.display())
            }
        }
    }
}

impl Error for ProcessingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProcessingError::Io(e) => Some(e),
            ProcessingError::File { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for ProcessingError {
    fn from(source: io::Error) -> Self {
        ProcessingError::Io(source)
    }
}

/// Word frequency table: word -> number of occurrences.
pub type WordCounts = HashMap<String, u64>;

/// Normalizes a raw token: lowercase, without leading/trailing punctuation.
/// Returns an empty string when nothing but punctuation remains.
fn normalize(token: &str) -> String {
    token
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// Counts word frequencies over lines of text. Pure and allocation-driven
/// only by the input — the unit tests exercise this directly.
pub fn count_words_in_lines(lines: impl IntoIterator<Item = String>) -> WordCounts {
    lines
        .into_iter()
        .flat_map(|line| {
            line.split_whitespace()
                .map(normalize)
                .filter(|word| !word.is_empty())
                .collect::<Vec<_>>()
        })
        .fold(HashMap::new(), |mut counts, word| {
            *counts.entry(word).or_insert(0) += 1;
            counts
        })
}

/// Merges `other` into `counts`, adding the frequencies together.
pub fn merge_counts(counts: &mut WordCounts, other: WordCounts) {
    for (word, n) in other {
        *counts.entry(word).or_insert(0) += n;
    }
}

/// Counts the words of a single file, reading it line by line.
pub fn count_file(path: &Path) -> Result<WordCounts, ProcessingError> {
    let file = File::open(path).map_err(|source| ProcessingError::File {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let lines = reader.lines().map_while(Result::ok);
    Ok(count_words_in_lines(lines))
}

/// Lists the `.txt` files in a directory (non-recursive, sorted by name).
pub fn collect_text_files(dir: &Path) -> Result<Vec<PathBuf>, ProcessingError> {
    let entries = fs::read_dir(dir).map_err(|source| ProcessingError::File {
        path: dir.to_path_buf(),
        source,
    })?;

    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "txt"))
        .collect();
    files.sort();
    Ok(files)
}

/// Counts words for a single file or for every `.txt` file in a directory.
pub fn count_path(input: &Path) -> Result<WordCounts, ProcessingError> {
    if !input.exists() {
        return Err(ProcessingError::NotFound {
            path: input.to_path_buf(),
        });
    }

    if input.is_file() {
        return count_file(input);
    }

    let files = collect_text_files(input)?;
    if files.is_empty() {
        return Err(ProcessingError::NoTextFiles {
            dir: input.to_path_buf(),
        });
    }

    let mut counts = HashMap::new();
    for file in files {
        merge_counts(&mut counts, count_file(&file)?);
    }
    Ok(counts)
}

/// Returns the `n` most frequent words, ties broken alphabetically.
pub fn top_n(counts: &WordCounts, n: usize) -> Vec<(&str, u64)> {
    let mut entries: Vec<(&str, u64)> = counts
        .iter()
        .map(|(word, &count)| (word.as_str(), count))
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    entries.truncate(n);
    entries
}

/// Renders a small plain-text report of the most frequent words.
pub fn format_report(counts: &WordCounts, n: usize) -> String {
    let total: u64 = counts.values().sum();
    let mut report = format!("{} unique words, {} total\n", counts.len(), total);
    for (rank, (word, count)) in top_n(counts, n).iter().enumerate() {
        report.push_str(&format!("{:>4}. {:<20} {}\n", rank + 1, word, count));
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines<const N: usize>(input: [&str; N]) -> Vec<String> {
        input.into_iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn counts_words_case_insensitively() {
        let counts = count_words_in_lines(lines(["Hello world", "hello, WORLD!"]));
        assert_eq!(counts["hello"], 2);
        assert_eq!(counts["world"], 2);
    }

    #[test]
    fn strips_surrounding_punctuation() {
        let counts = count_words_in_lines(lines(["(rust) — 'fast' ...rust..."]));
        assert_eq!(counts["rust"], 2);
        assert_eq!(counts["fast"], 1);
        // Pure punctuation tokens are dropped.
        assert!(!counts.contains_key("—"));
        assert!(!counts.contains_key(""));
    }

    #[test]
    fn empty_input_yields_empty_counts() {
        let counts = count_words_in_lines(lines([]));
        assert!(counts.is_empty());
    }

    #[test]
    fn merge_adds_frequencies() {
        let mut a = count_words_in_lines(lines(["one two two"]));
        let b = count_words_in_lines(lines(["two three"]));
        merge_counts(&mut a, b);
        assert_eq!(a["one"], 1);
        assert_eq!(a["two"], 3);
        assert_eq!(a["three"], 1);
    }

    #[test]
    fn top_n_sorts_by_count_then_word() {
        let counts = count_words_in_lines(lines(["b a a c c b d"]));
        let top = top_n(&counts, 3);
        assert_eq!(top, vec![("a", 2), ("b", 2), ("c", 2)]);
    }

    #[test]
    fn top_n_with_n_larger_than_vocab() {
        let counts = count_words_in_lines(lines(["only"]));
        assert_eq!(top_n(&counts, 10), vec![("only", 1)]);
    }

    #[test]
    fn report_contains_summary_and_rows() {
        let counts = count_words_in_lines(lines(["alpha beta alpha"]));
        let report = format_report(&counts, 5);
        assert!(report.contains("2 unique words, 3 total"));
        assert!(report.contains("alpha"));
        assert!(report.contains("beta"));
    }
}
