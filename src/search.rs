//! Search functionality with multiple modes

use serde::{Deserialize, Serialize};
use std::time::Instant;
use crate::tree::Document;
use crate::fts::{FtsIndex, SearchHit};

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
    /// Document name
    pub doc_name: String,
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

    /// Perform a simple search (placeholder for quick search without index)
    pub fn search(&self, query: &str) -> SearchResult {
        let start = Instant::now();

        SearchResult {
            documents: Vec::new(),
            paths: Vec::new(),
            query: query.to_string(),
            mode: self.mode,
            search_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Perform a flat search using FTS only
    pub fn search_flat(
        &self,
        query: &str,
        documents: &[Document],
        fts: &FtsIndex,
    ) -> anyhow::Result<SearchResult> {
        let start = Instant::now();

        // FTS search
        let hits = fts.search(query, 50)?;

        // Group hits by document
        let mut doc_results: std::collections::HashMap<String, DocumentResult> =
            std::collections::HashMap::new();

        for hit in hits {
            let doc = documents.iter().find(|d| d.doc_id == hit.doc_id);
            if let Some(doc) = doc {
                let node = doc.root.find_by_id(&hit.node_id);
                let (title, text, path) = if let Some(node) = node {
                    let path = doc.root.find_path_to(&hit.node_id)
                        .unwrap_or_else(|| vec![hit.node_id.clone()]);
                    (node.title.clone(), node.text.clone(), path)
                } else {
                    (hit.title.clone(), String::new(), vec![hit.node_id.clone()])
                };

                let node_result = NodeResult {
                    node_id: hit.node_id.clone(),
                    title,
                    text,
                    score: hit.score,
                    path,
                    line_number: None,
                };

                doc_results
                    .entry(doc.doc_name.clone())
                    .or_insert_with(|| DocumentResult {
                        doc_name: doc.doc_name.clone(),
                        nodes: Vec::new(),
                    })
                    .nodes
                    .push(node_result);
            }
        }

        let search_time_ms = start.elapsed().as_millis() as u64;

        Ok(SearchResult {
            documents: doc_results.into_values().collect(),
            paths: Vec::new(),
            query: query.to_string(),
            mode: SearchMode::Flat,
            search_time_ms,
        })
    }

    /// Perform a tree search with path aggregation
    pub fn search_tree(
        &self,
        query: &str,
        documents: &[Document],
        fts: &FtsIndex,
    ) -> anyhow::Result<SearchResult> {
        let start = Instant::now();

        // FTS search for anchors
        let anchors = fts.search(query, 20)?;

        // Build path results from anchors
        let mut paths: Vec<PathResult> = Vec::new();
        let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        for anchor in anchors {
            // Find the document
            let doc = documents.iter().find(|d| d.doc_id == anchor.doc_id);
            if let Some(doc) = doc {
                // Find path to anchor node
                if let Some(path_ids) = doc.root.find_path_to(&anchor.node_id) {
                    // Create path key for deduplication
                    let path_key = format!("{}:{}", doc.doc_id, path_ids.join("/"));
                    if seen_paths.contains(&path_key) {
                        continue;
                    }
                    seen_paths.insert(path_key);

                    // Build path nodes
                    let path_nodes: Vec<PathNode> = path_ids
                        .iter()
                        .filter_map(|node_id| {
                            doc.root.find_by_id(node_id).map(|node| PathNode {
                                node_id: node.node_id.clone(),
                                title: node.title.clone(),
                            })
                        })
                        .collect();

                    // Aggregate score (anchor score + path length bonus)
                    let path_score = anchor.score + (path_nodes.len() as f32 * 0.1);

                    // Build snippet from anchor
                    let snippet = if let Some(node) = doc.root.find_by_id(&anchor.node_id) {
                        let mut s = String::new();
                        if !node.title.is_empty() {
                            s.push_str(&node.title);
                            s.push_str("\n\n");
                        }
                        if !node.text.is_empty() {
                            s.push_str(&node.text.chars().take(200).collect::<String>());
                        }
                        s
                    } else {
                        anchor.title.clone()
                    };

                    paths.push(PathResult {
                        path: path_nodes,
                        score: path_score,
                        snippet,
                        doc_name: doc.doc_name.clone(),
                    });
                }
            }
        }

        // Sort by score descending
        paths.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let search_time_ms = start.elapsed().as_millis() as u64;

        Ok(SearchResult {
            documents: Vec::new(),
            paths,
            query: query.to_string(),
            mode: SearchMode::Tree,
            search_time_ms,
        })
    }

    /// Perform search with automatic mode selection
    pub fn search_auto(
        &self,
        query: &str,
        documents: &[Document],
        fts: &FtsIndex,
    ) -> anyhow::Result<SearchResult> {
        let mode = Self::auto_select_mode(documents);
        match mode {
            SearchMode::Tree => self.search_tree(query, documents, fts),
            _ => self.search_flat(query, documents, fts),
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