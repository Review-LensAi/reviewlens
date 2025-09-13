//! Logic for parsing diffs to identify changed files and hunks.

use crate::error::{EngineError, Result};
use patch::{Line as PatchLine, Patch};

/// Represents a single changed file in a diff.
#[derive(Debug)]
pub struct ChangedFile {
    pub path: String,
    pub hunks: Vec<Hunk>,
}

/// Represents a "hunk" or a contiguous block of changes in a file.
#[derive(Debug)]
pub struct Hunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<Line>,
}

/// Represents a single line in a hunk.
#[derive(Debug)]
pub enum Line {
    Added(String),
    Removed(String),
    Context(String),
}

/// Parses a raw diff string into a structured format using the `patch` crate.
///
/// # Arguments
///
/// * `diff_text` - A string containing the output of a `git diff` command.
///
/// # Returns
///
/// A `Result` containing a vector of `ChangedFile`s or an `EngineError`.
pub fn parse(diff_text: &str) -> Result<Vec<ChangedFile>> {
    if diff_text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut segment = String::new();

    for line in diff_text.lines() {
        if line.starts_with("diff --git ") {
            if !segment.is_empty() {
                files.push(parse_segment(&segment)?);
                segment.clear();
            }
        }
        segment.push_str(line);
        segment.push('\n');
    }

    if !segment.is_empty() {
        files.push(parse_segment(&segment)?);
    }

    Ok(files)
}

fn parse_segment(segment: &str) -> Result<ChangedFile> {
    let header_path = segment
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(3))
        .ok_or_else(|| EngineError::DiffParser("Malformed diff header".into()))?
        .trim_start_matches("b/")
        .to_string();

    let has_patch = segment.lines().any(|l| l.starts_with("--- "));
    let is_binary = segment
        .lines()
        .any(|l| l.starts_with("Binary files") || l.starts_with("GIT binary patch"));

    if !has_patch || is_binary {
        return Ok(ChangedFile {
            path: header_path,
            hunks: Vec::new(),
        });
    }

    let patches =
        Patch::from_multiple(segment).map_err(|e| EngineError::DiffParser(e.to_string()))?;
    let patch = patches
        .into_iter()
        .next()
        .ok_or_else(|| EngineError::DiffParser("No patch data found".into()))?;

    let path = patch.new.path.trim_start_matches("b/").to_string();
    let hunks = patch
        .hunks
        .into_iter()
        .map(|h| {
            let lines = h
                .lines
                .into_iter()
                .map(|l| match l {
                    PatchLine::Add(s) => Line::Added(s.to_string()),
                    PatchLine::Remove(s) => Line::Removed(s.to_string()),
                    PatchLine::Context(s) => Line::Context(s.to_string()),
                })
                .collect();
            Hunk {
                old_start: h.old_range.start as u32,
                old_lines: h.old_range.count as u32,
                new_start: h.new_range.start as u32,
                new_lines: h.new_range.count as u32,
                lines,
            }
        })
        .collect();

    Ok(ChangedFile { path, hunks })
}
