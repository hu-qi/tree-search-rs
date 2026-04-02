//! TreeSearch Rust - Tree-aware document search engine
//!
//! A pure Rust implementation of TreeSearch with Tantivy FTS support.

pub mod tree;
pub mod config;
pub mod search;
pub mod fts;
pub mod tokenizer;
pub mod indexer;
pub mod pathutil;
pub mod heuristics;
pub mod parsers;

pub use tree::{TreeNode, Document};
pub use search::{SearchResult, SearchMode};
pub use config::Config;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");