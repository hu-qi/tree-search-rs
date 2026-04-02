//! Search functionality with multiple modes

use serde::{Deserialize, Serialize};
use std::time::Instant;
use crate::tree::Document;

/// Search mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    Auto,
    Flat,
    Tree,
}

/// Result of a search operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching documents
    pub documents: Vec<DocumentResult>,
    /// Matching paths (for tree mode)
    pub paths: Vec<PathResult>,
    /// Original query
    pub query: String,
    /// Search mode used
    pub mode: SearchMode,
    /// Search time in milliseconds
    pub search_time_ms: u64,
}

/// Result for a single document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResult {
    /// Document name
    pub doc_name: String,
    /// Matching nodes
    pub nodes: Vec<NodeResult>,
}

/// Result for a single node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    /// Node identifier
    pub node_id: String,
    /// Node title
    pub title: String,
    /// Matching text snippet
    pub text: String,
    /// Relevance score
    pub score: f32,
    /// Path from root to this node
    pub path: Vec<String>,
    /// Line number if available
    pub line_number: Option<usize>,
}

/// Result for a path (tree mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    /// Path nodes
    pub path: Vec<PathNode>,
    /// Aggregate score
    pub score: f32,
    /// Text snippet
    pub snippet: String,
}

/// Node in a path result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    /// Node ID
    pub node_id: String,
    /// Node title
    pub title: String,
}

/// Search engine
pub struct SearchEngine {
    mode: SearchMode,
}

impl SearchEngine {
    /// Create a new search engine with the given mode
    pub fn new(mode: SearchMode) -> Self {
        Self { mode }
    }

    /// Perform a search
    pub fn search(&self, query: &str) -> SearchResult {
        let start = Instant::now();

        // TODO: Implement actual search logic
        // This is a placeholder that returns empty results

        SearchResult {
            documents: Vec::new(),
            paths: Vec::new(),
            query: query.to_string(),
            mode: self.mode,
            search_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Select appropriate search mode based on document characteristics
    pub fn auto_select_mode(documents: &[Document]) -> SearchMode {
        if documents.is_empty() {
            return SearchMode::Flat;
        }

        // Calculate percentage of documents that benefit from tree
        let tree_benefit_count = documents
            .iter()
            .filter(|d: &&Document| d.tree_benefit())
            .count();

        let ratio = tree_benefit_count as f32 / documents.len() as f32;

        // If >= 30% of docs benefit from tree, use tree mode
        if ratio >= 0.3 {
            SearchMode::Tree
        } else {
            SearchMode::Flat
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_select_mode_empty() {
        let mode = SearchEngine::auto_select_mode(&[]);
        assert_eq!(mode, SearchMode::Flat);
    }

    #[test]
    fn test_search_result_timing() {
        let engine = SearchEngine::new(SearchMode::Flat);
        let result = engine.search("test query");
        assert!(result.search_time_ms < 1000); // Should be very fast
    }
}