//! Plain text parser

use anyhow::Result;
use std::path::Path;

use crate::tree::{TreeNode, Document};
use super::Parser;

/// Plain text parser
pub struct TextParser {
    name: String,
}

impl TextParser {
    pub fn new() -> Self {
        Self {
            name: "text".to_string(),
        }
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for TextParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
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

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "text", path.to_path_buf());

        // Simple paragraph-based structure
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        let mut node_counter = 0;

        for para in paragraphs {
            let trimmed = para.trim();
            if trimmed.is_empty() {
                continue;
            }

            node_counter += 1;
            let node_id = format!("{}:para:{}", doc_id, node_counter);

            // Use first line as title
            let first_line = trimmed.lines().next().unwrap_or("");
            let title = if first_line.len() > 50 {
                format!("{}...", &first_line[..50])
            } else {
                first_line.to_string()
            };

            let mut node = TreeNode::new(node_id, title);
            node.text = trimmed.to_string();
            doc.root.children.push(node);
        }

        Ok(doc)
    }
}