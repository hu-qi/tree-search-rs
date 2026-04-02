//! CSV parser

use anyhow::Result;
use std::path::Path;

use crate::tree::{TreeNode, Document};
use super::Parser;

/// CSV parser
pub struct CsvParser {
    name: String,
}

impl CsvParser {
    pub fn new() -> Self {
        Self {
            name: "csv".to_string(),
        }
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for CsvParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["csv"]
    }

    fn tree_benefit(&self) -> bool {
        false
    }

    fn parse(&self, content: &str, path: &Path) -> Result<Document> {
        let doc_id = uuid::Uuid::new_v4().to_string();
        let doc_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
            .to_string();

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "csv", path.to_path_buf());

        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

        let mut node_counter = 0;

        for result in reader.records() {
            let record = result?;
            node_counter += 1;
            let node_id = format!("{}:row:{}", doc_id, node_counter);

            let mut node = TreeNode::new(node_id, format!("Row {}", node_counter));
            let mut text_parts = Vec::new();

            for (i, value) in record.iter().enumerate() {
                if i < headers.len() {
                    text_parts.push(format!("{}: {}", headers[i], value));
                }
            }

            node.text = text_parts.join(", ");
            doc.root.children.push(node);
        }

        // Store headers in metadata
        doc.root.metadata.insert("headers".to_string(), headers.join(", "));
        doc.root.metadata.insert("columns".to_string(), headers.len().to_string());
        doc.root.metadata.insert("rows".to_string(), node_counter.to_string());

        Ok(doc)
    }
}