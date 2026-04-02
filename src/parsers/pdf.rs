//! PDF parser using lopdf

use anyhow::{Result, Context};
use std::path::Path;
use lopdf::Document as PdfDocument;

use crate::tree::{TreeNode, Document};
use super::Parser;

/// PDF parser
pub struct PdfParser {
    name: String,
}

impl PdfParser {
    pub fn new() -> Self {
        Self {
            name: "pdf".to_string(),
        }
    }
}

impl Default for PdfParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for PdfParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["pdf"]
    }

    fn tree_benefit(&self) -> bool {
        false
    }

    fn parse(&self, content: &str, path: &Path) -> Result<Document> {
        // Note: content is ignored for PDF, we read from file
        let _ = content;
        
        let doc_id = uuid::Uuid::new_v4().to_string();
        let doc_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
            .to_string();

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "pdf", path.to_path_buf());

        // Load PDF from file
        let pdf = PdfDocument::load(path)
            .context("Failed to load PDF file")?;

        // Get all page numbers
        let pages: Vec<u32> = pdf.get_pages().keys().cloned().collect();

        // Extract text from all pages
        for (idx, &page_num) in pages.iter().enumerate() {
            let page_node_id = format!("{}:page:{}", doc_id, page_num);
            
            // Extract text from this page
            match pdf.extract_text(&[page_num]) {
                Ok(page_text) => {
                    let trimmed = page_text.trim();
                    if !trimmed.is_empty() {
                        let title = format!("Page {}", idx + 1);
                        let mut node = TreeNode::new(page_node_id, title);
                        node.text = trimmed.to_string();
                        node.metadata.insert("page".to_string(), page_num.to_string());
                        doc.root.children.push(node);
                    }
                }
                Err(_) => {
                    // Skip pages that can't be extracted
                    continue;
                }
            }
        }

        Ok(doc)
    }
}