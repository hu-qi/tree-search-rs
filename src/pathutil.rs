//! Path utilities for file discovery and filtering

use anyhow::Result;
use std::path::{Path, PathBuf};
use ignore::WalkBuilder;

/// File type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Markdown,
    Text,
    Python,
    JavaScript,
    Java,
    Go,
    Rust,
    Cpp,
    Json,
    Jsonl,
    Csv,
    Xml,
    Html,
    Pdf,
    Docx,
    Unknown,
}

impl FileType {
    /// Detect file type from extension
    pub fn from_path(path: &Path) -> Self {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match ext.as_deref() {
            Some("md") => Self::Markdown,
            Some("txt") => Self::Text,
            Some("py") => Self::Python,
            Some("js" | "jsx" | "ts" | "tsx") => Self::JavaScript,
            Some("java") => Self::Java,
            Some("go") => Self::Go,
            Some("rs") => Self::Rust,
            Some("cpp" | "cc" | "cxx" | "c" | "h" | "hpp") => Self::Cpp,
            Some("json") => Self::Json,
            Some("jsonl") => Self::Jsonl,
            Some("csv") => Self::Csv,
            Some("xml") => Self::Xml,
            Some("html" | "htm") => Self::Html,
            Some("pdf") => Self::Pdf,
            Some("docx") => Self::Docx,
            _ => Self::Unknown,
        }
    }

    /// Check if this file type supports tree structure
    pub fn tree_benefit(&self) -> bool {
        matches!(
            self,
            Self::Markdown | Self::Json | Self::Xml | Self::Html
        )
    }

    /// Get supported extensions
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Markdown => &["md"],
            Self::Text => &["txt"],
            Self::Python => &["py"],
            Self::JavaScript => &["js", "jsx", "ts", "tsx"],
            Self::Java => &["java"],
            Self::Go => &["go"],
            Self::Rust => &["rs"],
            Self::Cpp => &["cpp", "cc", "cxx", "c", "h", "hpp"],
            Self::Json => &["json"],
            Self::Jsonl => &["jsonl"],
            Self::Csv => &["csv"],
            Self::Xml => &["xml"],
            Self::Html => &["html", "htm"],
            Self::Pdf => &["pdf"],
            Self::Docx => &["docx"],
            Self::Unknown => &[],
        }
    }
}

/// Path discovery options
#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    /// Include patterns (glob)
    pub include: Vec<String>,
    /// Exclude patterns (glob)
    pub exclude: Vec<String>,
    /// Follow symlinks
    pub follow_links: bool,
    /// Max depth
    pub max_depth: usize,
    /// Respect .gitignore
    pub respect_gitignore: bool,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            include: vec!["**/*".to_string()],
            exclude: vec![
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/__pycache__/**".to_string(),
            ],
            follow_links: false,
            max_depth: 10,
            respect_gitignore: true,
        }
    }
}

/// Discover files in a directory using ripgrep's ignore library
///
/// This is significantly faster than walkdir because:
/// 1. Parallel traversal by default
/// 2. Built-in .gitignore support
/// 3. Optimized filtering
pub fn discover_files(root: &Path, options: &DiscoveryOptions) -> Result<Vec<PathBuf>> {
    use ignore::overrides::OverrideBuilder;

    let mut files = Vec::new();

    let mut builder = WalkBuilder::new(root);
    builder
        .follow_links(options.follow_links)
        .max_depth(Some(options.max_depth))
        .git_ignore(options.respect_gitignore)
        .git_global(options.respect_gitignore)
        .git_exclude(options.respect_gitignore)
        .ignore(options.respect_gitignore)
        .hidden(false)  // Include hidden files
        .threads(num_cpus::get());  // Parallel traversal

    // Add custom ignore patterns using OverrideBuilder
    // This correctly handles glob patterns instead of treating them as literal file paths
    if !options.exclude.is_empty() {
        let mut override_builder = OverrideBuilder::new(root);
        for pattern in &options.exclude {
            // Prefix with '!' to exclude files matching the pattern
            override_builder.add(&format!("!{}", pattern))
                .map_err(|e| anyhow::anyhow!("Invalid ignore pattern '{}': {}", pattern, e))?;
        }
        let overrides = override_builder.build()
            .map_err(|e| anyhow::anyhow!("Failed to build override rules: {}", e))?;
        builder.overrides(overrides);
    }

    for result in builder.build() {
        match result {
            Ok(entry) => {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    let path = entry.path();

                    // Check if file type is known
                    let ft = FileType::from_path(path);
                    if ft != FileType::Unknown {
                        files.push(path.to_path_buf());
                    }
                }
            }
            Err(err) => {
                // Log error but continue
                tracing::debug!("Error walking: {}", err);
            }
        }
    }

    Ok(files)
}

/// Fast file search using ripgrep's search capabilities
///
/// This uses the `grep` crate for ultra-fast regex search
pub fn fast_search_files(root: &Path, pattern: &str, options: &DiscoveryOptions) -> Result<Vec<PathBuf>> {
    use regex::Regex;

    let re = Regex::new(pattern)?;
    let files = discover_files(root, options)?;

    let matches: Vec<PathBuf> = files
        .into_iter()
        .filter(|path| re.is_match(&path.to_string_lossy()))
        .collect();

    Ok(matches)
}

/// Check if a path matches a glob pattern
pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();
    // Simple glob matching using glob crate
    match glob::Pattern::new(pattern) {
        Ok(p) => p.matches(&path_str),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        assert_eq!(
            FileType::from_path(Path::new("test.md")),
            FileType::Markdown
        );
        assert_eq!(
            FileType::from_path(Path::new("test.py")),
            FileType::Python
        );
        assert_eq!(
            FileType::from_path(Path::new("test.rs")),
            FileType::Rust
        );
    }

    #[test]
    fn test_tree_benefit() {
        assert!(FileType::Markdown.tree_benefit());
        assert!(FileType::Json.tree_benefit());
        assert!(!FileType::Python.tree_benefit());
    }
}