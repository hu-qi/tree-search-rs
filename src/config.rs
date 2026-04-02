//! Configuration management

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database/index directory path
    pub db_path: PathBuf,
    /// Maximum nodes per document
    pub max_nodes_per_doc: usize,
    /// Maximum files to index
    pub max_files: usize,
    /// Search mode (auto, flat, tree)
    pub search_mode: SearchModeConfig,
    /// Generate node summaries
    pub add_description: bool,
    /// Index directory for pre-built indexes
    pub index_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from(".treesearch.db"),
            max_nodes_per_doc: 1000,
            max_files: 10000,
            search_mode: SearchModeConfig::Auto,
            add_description: false,
            index_dir: None,
        }
    }
}

impl Config {
    /// Create config with overrides from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(path) = std::env::var("TREESEARCH_DB_PATH") {
            config.db_path = PathBuf::from(path);
        }
        if let Ok(max_nodes) = std::env::var("TREESEARCH_MAX_NODES") {
            if let Ok(n) = max_nodes.parse() {
                config.max_nodes_per_doc = n;
            }
        }
        if let Ok(max_files) = std::env::var("TREESEARCH_MAX_FILES") {
            if let Ok(n) = max_files.parse() {
                config.max_files = n;
            }
        }
        if let Ok(mode) = std::env::var("TREESEARCH_SEARCH_MODE") {
            config.search_mode = match mode.to_lowercase().as_str() {
                "flat" => SearchModeConfig::Flat,
                "tree" => SearchModeConfig::Tree,
                _ => SearchModeConfig::Auto,
            };
        }

        config
    }

    /// Create config for a specific project directory
    pub fn for_project(project_dir: impl Into<PathBuf>) -> Self {
        let mut config = Self::from_env();
        config.db_path = project_dir.into().join(".treesearch.db");
        config
    }
}

/// Search mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchModeConfig {
    Auto,
    Flat,
    Tree,
}

impl std::fmt::Display for SearchModeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Flat => write!(f, "flat"),
            Self::Tree => write!(f, "tree"),
        }
    }
}

impl std::str::FromStr for SearchModeConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "flat" => Ok(Self::Flat),
            "tree" => Ok(Self::Tree),
            _ => Err(format!("Invalid search mode: {}", s)),
        }
    }
}