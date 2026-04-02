//! Markdown parser

use anyhow::Result;
use std::path::Path;
use pulldown_cmark::{Parser as CmarkParser, Event, Tag, HeadingLevel};

use crate::tree::{TreeNode, Document};
use super::Parser;

/// Markdown parser
pub struct MarkdownParser {
    name: String,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            name: "markdown".to_string(),
        }
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for MarkdownParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["md"]
    }

    fn tree_benefit(&self) -> bool {
        true
    }

    fn parse(&self, content: &str, path: &Path) -> Result<Document> {
        let doc_id = uuid::Uuid::new_v4().to_string();
        let doc_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document")
            .to_string();

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "markdown", path.to_path_buf());

        // Parse markdown
        let parser = CmarkParser::new(content);
        let mut current_node = TreeNode::new(format!("{}:root", doc_id), doc_name);
        let mut node_stack: Vec<TreeNode> = vec![];
        let mut current_text = String::new();
        let mut current_code = String::new();
        let mut in_code_block = false;
        let mut node_counter = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Heading(level)) => {
                    // Save current node content
                    if !current_text.trim().is_empty() {
                        current_node.text = current_text.trim().to_string();
                        current_text.clear();
                    }

                    // Determine heading depth
                    let depth = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };

                    // Adjust stack to correct depth
                    while node_stack.len() >= depth {
                        let node = node_stack.pop().unwrap();
                        if let Some(parent) = node_stack.last_mut() {
                            parent.children.push(node);
                        } else {
                            doc.root.children.push(node);
                        }
                    }
                }
                Event::End(Tag::Heading(_)) => {
                    // Create new node for this heading
                    node_counter += 1;
                    let node_id = format!("{}:node:{}", doc_id, node_counter);
                    let title = current_text.trim().to_string();
                    current_text.clear();

                    let node = TreeNode::new(node_id, title);
                    node_stack.push(node);
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    in_code_block = true;
                    current_code.clear();
                }
                Event::End(Tag::CodeBlock(_)) => {
                    in_code_block = false;
                    if let Some(node) = node_stack.last_mut() {
                        node.code = Some(current_code.trim().to_string());
                    }
                    current_code.clear();
                }
                Event::Text(text) => {
                    if in_code_block {
                        current_code.push_str(&text);
                    } else {
                        current_text.push_str(&text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !in_code_block {
                        current_text.push(' ');
                    }
                }
                _ => {}
            }
        }

        // Save remaining content
        if !current_text.trim().is_empty() {
            if let Some(node) = node_stack.last_mut() {
                node.text = current_text.trim().to_string();
            } else {
                doc.root.text = current_text.trim().to_string();
            }
        }

        // Pop remaining nodes
        while let Some(node) = node_stack.pop() {
            if let Some(parent) = node_stack.last_mut() {
                parent.children.push(node);
            } else {
                doc.root.children.push(node);
            }
        }

        Ok(doc)
    }
}