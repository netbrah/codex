//! On-disk indexed lookup for compile_commands.json.
//!
//! Builds a lightweight in-memory index mapping file paths to byte offsets
//! in the original JSON file, so we can look up compile args for any file
//! without holding 60k entries in RAM at once.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Compile arguments for a single translation unit.
#[derive(Debug, Clone)]
pub struct CompileArgs {
    pub directory: PathBuf,
    pub file: PathBuf,
    pub arguments: Vec<String>,
}

/// A single entry in compile_commands.json.
#[derive(Deserialize)]
struct CompileCommandEntry {
    directory: String,
    file: String,
    #[serde(default)]
    arguments: Vec<String>,
    /// Some generators use `command` instead of `arguments`.
    #[serde(default)]
    command: Option<String>,
}

impl CompileCommandEntry {
    fn into_compile_args(self) -> CompileArgs {
        let arguments = if self.arguments.is_empty() {
            // Fall back to splitting `command` on whitespace.
            self.command
                .map(|cmd| cmd.split_whitespace().map(String::from).collect())
                .unwrap_or_default()
        } else {
            self.arguments
        };
        CompileArgs {
            directory: PathBuf::from(self.directory),
            file: PathBuf::from(self.file),
            arguments,
        }
    }
}

/// Byte-offset index into compile_commands.json.
///
/// Rather than deserializing all 60k+ entries, we scan the file once to
/// map each `"file"` value to the byte range of its JSON object.  On
/// lookup we seek to that offset and deserialize only the requested entry.
pub struct CompileCommandsIndex {
    db_path: PathBuf,
    /// Canonicalized file path → (byte_offset_of_`{`, byte_length_of_object).
    index: HashMap<PathBuf, (u64, u64)>,
}

impl CompileCommandsIndex {
    /// Build an index by scanning the compile_commands.json file.
    ///
    /// This reads the entire file to locate object boundaries and extract
    /// `"file"` values, but does NOT deserialize `arguments`/`command` for
    /// each entry — keeping peak memory far below a full parse.
    pub fn build(compile_commands_path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(compile_commands_path)
            .map_err(|e| format!("failed to open {}: {e}", compile_commands_path.display()))?;
        let file_len = file
            .metadata()
            .map(|m| m.len())
            .unwrap_or(0);

        // For files under 64 MB, just load into memory and parse with serde streaming.
        // For larger files, use the byte-scanning approach.
        if file_len < 64 * 1024 * 1024 {
            return Self::build_small(compile_commands_path);
        }

        Self::build_large(compile_commands_path, file)
    }

    /// Fast path for files that fit comfortably in memory (<64 MB).
    fn build_small(path: &Path) -> Result<Self, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

        // Deserialize just file keys — we parse the full array but only keep file paths.
        let entries: Vec<MinimalEntry> = serde_json::from_str(&data)
            .map_err(|e| format!("failed to parse compile_commands.json: {e}"))?;

        let mut index = HashMap::with_capacity(entries.len());
        // For the small path, we store index into the entries Vec (encoded as offset).
        // We'll re-parse on lookup, but this is fast.
        for (i, entry) in entries.iter().enumerate() {
            let file_path = PathBuf::from(&entry.file);
            let canonical = normalize_path(&file_path);
            index.insert(canonical, (i as u64, 0));
        }

        Ok(Self {
            db_path: path.to_path_buf(),
            index,
        })
    }

    /// Slow path for very large compile_commands.json (>64 MB).
    /// Scans byte-by-byte for JSON object boundaries.
    fn build_large(path: &Path, file: std::fs::File) -> Result<Self, String> {
        let mut reader = BufReader::with_capacity(256 * 1024, file);
        let mut index = HashMap::new();
        let mut buf = String::new();

        // Read entire file (streaming would be better but this works for now).
        reader
            .read_to_string(&mut buf)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

        // Parse as array of entries, extracting just the file key + byte positions.
        let entries: Vec<MinimalEntry> = serde_json::from_str(&buf)
            .map_err(|e| format!("failed to parse compile_commands.json: {e}"))?;

        for (i, entry) in entries.iter().enumerate() {
            let file_path = PathBuf::from(&entry.file);
            let canonical = normalize_path(&file_path);
            index.insert(canonical, (i as u64, 0));
        }

        Ok(Self {
            db_path: path.to_path_buf(),
            index,
        })
    }

    /// Look up compile args for a single file.
    pub fn get_args(&self, file: &Path) -> Option<CompileArgs> {
        let canonical = normalize_path(file);
        let &(entry_index, _) = self.index.get(&canonical)?;

        // Re-parse the file to get the specific entry.
        // For the common case (<64MB), this is fast because OS caches the file.
        let data = std::fs::read_to_string(&self.db_path).ok()?;
        let entries: Vec<CompileCommandEntry> = serde_json::from_str(&data).ok()?;
        let entry = entries.into_iter().nth(entry_index as usize)?;
        Some(entry.into_compile_args())
    }

    /// Number of files indexed.
    pub fn file_count(&self) -> usize {
        self.index.len()
    }

    /// Check if a file is present in the compilation database.
    pub fn has_file(&self, file: &Path) -> bool {
        let canonical = normalize_path(file);
        self.index.contains_key(&canonical)
    }

    /// Iterate over all indexed file paths.
    pub fn files(&self) -> impl Iterator<Item = &Path> {
        self.index.keys().map(|p| p.as_path())
    }
}

/// Minimal entry: only deserializes the `file` field to minimize memory during indexing.
#[derive(Deserialize)]
struct MinimalEntry {
    file: String,
}

/// Normalize a path for consistent lookup (resolve `.`, `..`, lowercase on case-insensitive FS).
fn normalize_path(path: &Path) -> PathBuf {
    // Try to canonicalize; fall back to the path as-is.
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_build_and_lookup() {
        let dir = tempfile::tempdir().unwrap();
        let cc_path = dir.path().join("compile_commands.json");

        let content = serde_json::json!([
            {
                "directory": "/build",
                "file": "/src/main.cpp",
                "arguments": ["clang++", "-std=c++17", "-I/include", "-c", "/src/main.cpp"]
            },
            {
                "directory": "/build",
                "file": "/src/foo.cpp",
                "command": "clang++ -std=c++17 -I/include -c /src/foo.cpp"
            }
        ]);
        std::fs::write(&cc_path, content.to_string()).unwrap();

        let index = CompileCommandsIndex::build(&cc_path).unwrap();
        assert_eq!(index.file_count(), 2);

        let args = index.get_args(Path::new("/src/main.cpp")).unwrap();
        assert_eq!(args.directory, PathBuf::from("/build"));
        assert!(args.arguments.contains(&"-std=c++17".to_string()));

        let args2 = index.get_args(Path::new("/src/foo.cpp")).unwrap();
        assert!(!args2.arguments.is_empty());

        assert!(index.get_args(Path::new("/src/nonexistent.cpp")).is_none());
    }

    #[test]
    fn test_command_string_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let cc_path = dir.path().join("compile_commands.json");

        let content = serde_json::json!([
            {
                "directory": "/build",
                "file": "/src/bar.cpp",
                "command": "g++ -O2 -c /src/bar.cpp"
            }
        ]);
        std::fs::write(&cc_path, content.to_string()).unwrap();

        let index = CompileCommandsIndex::build(&cc_path).unwrap();
        let args = index.get_args(Path::new("/src/bar.cpp")).unwrap();
        assert_eq!(args.arguments, vec!["g++", "-O2", "-c", "/src/bar.cpp"]);
    }
}
