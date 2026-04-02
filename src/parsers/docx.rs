//! DOCX parser using docx-rs

use anyhow::{Result, Context};
use std::path::Path;
use std::fs;

use crate::tree::{TreeNode, Document};
use super::Parser;

/// DOCX parser
pub struct DocxParser {
    name: String,
}

impl DocxParser {
    pub fn new() -> Self {
        Self {
            name: "docx".to_string(),
        }
    }
}

impl Default for DocxParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for DocxParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["docx"]
    }

    fn tree_benefit(&self) -> bool {
        true // DOCX has heading structure
    }

    fn parse(&self, content: &str, path: &Path) -> Result<Document> {
        // Note: content is ignored for DOCX, we read from file
        let _ = content;
        
        let doc_id = uuid::Uuid::new_v4().to_string();
        let doc_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
            .to_string();

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "docx", path.to_path_buf());

        // Load DOCX file
        let bytes = fs::read(path)
            .context("Failed to read DOCX file")?;
        let docx = docx_rs::read_docx(&bytes)
            .context("Failed to parse DOCX file")?;

        // Extract content from document
        let mut para_counter = 0;
        let mut current_heading: Option<TreeNode> = None;
        let mut heading_counter = 0;

        for child in docx.document.children {
            match child {
                // Paragraph
                docx_rs::DocumentChild::Paragraph(para) => {
                    para_counter += 1;
                    
                    // Extract text from paragraph
                    let mut para_text = String::new();
                    for para_child in &para.children {
                        if let docx_rs::ParagraphChild::Run(run) = para_child {
                            for run_child in &run.children {
                                if let docx_rs::RunChild::Text(text) = run_child {
                                    para_text.push_str(&text.text);
                                }
                            }
                        }
                    }

                    let trimmed = para_text.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Check if this is a heading (based on style)
                    let is_heading = para.property.style
                        .as_ref()
                        .map(|s| {
                            let style = s.val.to_lowercase();
                            style.starts_with("heading") || 
                            style.starts_with("标题") ||
                            style.starts_with("h1") ||
                            style.starts_with("h2") ||
                            style.starts_with("h3")
                        })
                        .unwrap_or(false);

                    if is_heading {
                        // Save previous heading if exists
                        if let Some(heading) = current_heading.take() {
                            doc.root.children.push(heading);
                        }
                        
                        // Start new heading section
                        heading_counter += 1;
                        let node_id = format!("{}:heading:{}", doc_id, heading_counter);
                        let mut node = TreeNode::new(node_id, trimmed.to_string());
                        node.metadata.insert("type".to_string(), "heading".to_string());
                        current_heading = Some(node);
                    } else {
                        // Regular paragraph
                        let node_id = format!("{}:para:{}", doc_id, para_counter);
                        let mut node = TreeNode::new(node_id, "Paragraph");
                        node.text = trimmed.to_string();

                        if let Some(ref mut heading) = current_heading {
                            heading.children.push(node);
                        } else {
                            doc.root.children.push(node);
                        }
                    }
                }
                // Table
                docx_rs::DocumentChild::Table(table) => {
                    para_counter += 1;
                    
                    // Extract table content
                    let mut table_text = String::new();

                    for row in &table.rows {
                        if let docx_rs::TableChild::TableRow(table_row) = row {
                            let mut row_text = Vec::new();
                            
                            for cell in &table_row.cells {
                                if let docx_rs::TableRowChild::TableCell(cell) = cell {
                                    let mut cell_text = String::new();
                                    for child in &cell.children {
                                        if let docx_rs::TableCellContent::Paragraph(para) = child {
                                            for para_child in &para.children {
                                                if let docx_rs::ParagraphChild::Run(run) = para_child {
                                                    for run_child in &run.children {
                                                        if let docx_rs::RunChild::Text(text) = run_child {
                                                            cell_text.push_str(&text.text);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    row_text.push(cell_text);
                                }
                            }
                            
                            table_text.push_str(&row_text.join(" | "));
                            table_text.push('\n');
                        }
                    }

                    if !table_text.trim().is_empty() {
                        let node_id = format!("{}:table:{}", doc_id, para_counter);
                        let mut node = TreeNode::new(node_id, "Table");
                        node.text = table_text.trim().to_string();
                        node.metadata.insert("type".to_string(), "table".to_string());

                        if let Some(ref mut heading) = current_heading {
                            heading.children.push(node);
                        } else {
                            doc.root.children.push(node);
                        }
                    }
                }
                _ => {}
            }
        }

        // Save last heading if exists
        if let Some(heading) = current_heading {
            doc.root.children.push(heading);
        }

        Ok(doc)
    }
}
