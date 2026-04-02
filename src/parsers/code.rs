//! Code parser using tree-sitter

use anyhow::Result;
use std::path::Path;

use crate::tree::{TreeNode, Document};
use crate::pathutil::FileType;
use super::Parser;

/// Code parser
pub struct CodeParser {
    name: String,
}

impl CodeParser {
    pub fn new() -> Self {
        Self {
            name: "code".to_string(),
        }
    }

    /// Get language from file type
    fn get_language(&self, ft: FileType) -> Option<&'static str> {
        match ft {
            FileType::Python => Some("python"),
            FileType::JavaScript => Some("javascript"),
            FileType::Java => Some("java"),
            FileType::Go => Some("go"),
            FileType::Rust => Some("rust"),
            FileType::Cpp => Some("cpp"),
            _ => None,
        }
    }
}

impl Default for CodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for CodeParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["py", "js", "jsx", "ts", "tsx", "java", "go", "rs", "cpp", "cc", "cxx", "c", "h", "hpp"]
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

        let ft = FileType::from_path(path);
        let source_type = self.get_language(ft).unwrap_or("code");

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), source_type, path.to_path_buf());

        // Simple function/class extraction using regex
        // TODO: Use tree-sitter for proper AST parsing
        let mut node_counter = 0;

        // Python functions/classes
        if ft == FileType::Python {
            let lines: Vec<&str> = content.lines().collect();
            let mut current_func = String::new();
            let mut in_func = false;
            let mut indent = 0;

            for line in &lines {
                let trimmed = line.trim();

                if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                    // Save previous function
                    if !current_func.is_empty() {
                        node_counter += 1;
                        let node_id = format!("{}:func:{}", doc_id, node_counter);
                        let first_line = current_func.lines().next().unwrap_or("");
                        let title = first_line.split('(').next().unwrap_or(first_line).to_string();

                        let mut node = TreeNode::new(node_id, title);
                        node.code = Some(current_func.clone());
                        doc.root.children.push(node);
                    }

                    current_func = format!("{}\n", line);
                    in_func = true;
                    indent = line.len() - trimmed.len();
                } else if in_func {
                    let line_indent = line.len() - trimmed.len();
                    if !trimmed.is_empty() && line_indent <= indent {
                        // End of function
                        in_func = false;
                        node_counter += 1;
                        let node_id = format!("{}:func:{}", doc_id, node_counter);
                        let first_line = current_func.lines().next().unwrap_or("");
                        let title = first_line.split('(').next().unwrap_or(first_line).to_string();

                        let mut node = TreeNode::new(node_id, title);
                        node.code = Some(current_func.clone());
                        doc.root.children.push(node);
                        current_func.clear();
                    } else {
                        current_func.push_str(&format!("{}\n", line));
                    }
                }
            }

            // Save last function
            if !current_func.is_empty() {
                node_counter += 1;
                let node_id = format!("{}:func:{}", doc_id, node_counter);
                let first_line = current_func.lines().next().unwrap_or("");
                let title = first_line.split('(').next().unwrap_or(first_line).to_string();

                let mut node = TreeNode::new(node_id, title);
                node.code = Some(current_func);
                doc.root.children.push(node);
            }
        } else {
            // For other languages, just store the whole file
            doc.root.code = Some(content.to_string());
            doc.root.text = content.to_string();
        }

        Ok(doc)
    }
}