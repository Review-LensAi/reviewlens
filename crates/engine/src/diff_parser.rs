//! Logic for parsing diffs to identify changed files and hunks.

use crate::error::Result;

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

/// Parses a raw diff string into a structured format.
///
/// # Arguments
///
/// * `diff_text` - A string containing the output of a `git diff` command.
///
/// # Returns
///
/// A `Result` containing a vector of `ChangedFile`s or a `DiffParserError`.
pub fn parse(diff_text: &str) -> Result<Vec<ChangedFile>> {
    // A real implementation would use a proper diff parsing library (e.g., `diffy` or similar)
    // or parse the unified diff format manually.
    if diff_text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut lines = diff_text.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("diff --git ") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() < 4 {
                return Err(EngineError::DiffParser("Malformed diff header".into()));
            }
            let path = tokens[3].trim_start_matches("b/").to_string();

            // Advance to the file markers "---" and "+++"
            while let Some(l) = lines.next() {
                if l.starts_with("--- ") {
                    break;
                }
            }

            let plus_line = lines
                .next()
                .ok_or_else(|| EngineError::DiffParser("Missing +++ line".into()))?;
            if !plus_line.starts_with("+++ ") {
                return Err(EngineError::DiffParser("Missing +++ line".into()));
            }

            let mut hunks = Vec::new();
            while let Some(peek) = lines.peek() {
                if peek.starts_with("diff --git ") {
                    break;
                }
                if peek.starts_with("@@") {
                    let header = lines.next().unwrap();
                    let hunk = parse_hunk(header, &mut lines)?;
                    hunks.push(hunk);
                } else {
                    // Skip any other metadata lines
                    lines.next();
                }
            }

            files.push(ChangedFile { path, hunks });
        }
    }

    Ok(files)
}

fn parse_hunk<'a, I>(header: &str, lines: &mut std::iter::Peekable<I>) -> Result<Hunk>
where
    I: Iterator<Item = &'a str>,
{
    // Header example: "@@ -1,3 +1,3 @@"
    let header = header.trim();
    if !header.starts_with("@@") {
        return Err(EngineError::DiffParser("Invalid hunk header".into()));
    }

    let header = header
        .trim_start_matches("@@")
        .trim_end_matches("@@")
        .trim();

    let mut parts = header.split(' ');
    let old_part = parts
        .next()
        .ok_or_else(|| EngineError::DiffParser("Missing old range".into()))?;
    let new_part = parts
        .next()
        .ok_or_else(|| EngineError::DiffParser("Missing new range".into()))?;

    let (old_start, old_lines) = parse_range(old_part.trim_start_matches('-'))?;
    let (new_start, new_lines) = parse_range(new_part.trim_start_matches('+'))?;

    let mut hunk_lines = Vec::new();
    while let Some(peek) = lines.peek() {
        if peek.starts_with("@@") || peek.starts_with("diff --git ") {
            break;
        }
        let l = lines.next().unwrap();
        let mut chars = l.chars();
        match chars.next() {
            Some('+') => hunk_lines.push(Line::Added(chars.as_str().to_string())),
            Some('-') => hunk_lines.push(Line::Removed(chars.as_str().to_string())),
            Some(' ') => hunk_lines.push(Line::Context(chars.as_str().to_string())),
            Some('\\') => hunk_lines.push(Line::Context(l.to_string())),
            Some(other) => {
                return Err(EngineError::DiffParser(format!(
                    "Invalid line in hunk: starts with '{}'",
                    other
                )))
            }
            None => hunk_lines.push(Line::Context(String::new())),
        }
    }

    Ok(Hunk {
        old_start,
        old_lines,
        new_start,
        new_lines,
        lines: hunk_lines,
    })
}

fn parse_range(range: &str) -> Result<(u32, u32)> {
    let mut it = range.split(',');
    let start = it
        .next()
        .ok_or_else(|| EngineError::DiffParser("Missing range start".into()))?
        .parse::<u32>()
        .map_err(|_| EngineError::DiffParser("Invalid range start".into()))?;
    let lines = it
        .next()
        .map(|s| {
            s.parse::<u32>()
                .map_err(|_| EngineError::DiffParser("Invalid range length".into()))
        })
        .transpose()?
        .unwrap_or(1);
    Ok((start, lines))
}
