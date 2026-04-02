//! JSON parser

use anyhow::Result;
use std::path::Path;
use serde_json::Value;

use crate::tree::{TreeNode, Document};
use super::Parser;

/// JSON parser
pub struct JsonParser {
    name: String,
}

impl JsonParser {
    pub fn new() -> Self {
        Self {
            name: "json".to_string(),
        }
    }

    fn value_to_node(&self, key: &str, value: &Value, doc_id: &str, counter: &mut usize) -> TreeNode {
        *counter += 1;
        let node_id = format!("{}:key:{}", doc_id, counter);

        let mut node = TreeNode::new(node_id, key.to_string());

        match value {
            Value::Object(obj) => {
                for (k, v) in obj {
                    let child = self.value_to_node(k, v, doc_id, counter);
                    node.children.push(child);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let child = self.value_to_node(&format!("[{}]", i), v, doc_id, counter);
                    node.children.push(child);
                }
            }
            Value::String(s) => {
                node.text = s.clone();
            }
            Value::Number(n) => {
                node.text = n.to_string();
            }
            Value::Bool(b) => {
                node.text = b.to_string();
            }
            Value::Null => {
                node.text = "null".to_string();
            }
        }

        node
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for JsonParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["json", "jsonl"]
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

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "json", path.to_path_buf());

        let value: Value = serde_json::from_str(content)?;
        let mut counter = 0;

        match &value {
            Value::Object(obj) => {
                for (k, v) in obj {
                    let node = self.value_to_node(k, v, &doc_id, &mut counter);
                    doc.root.children.push(node);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let node = self.value_to_node(&format!("[{}]", i), v, &doc_id, &mut counter);
                    doc.root.children.push(node);
                }
            }
            _ => {
                doc.root.text = content.to_string();
            }
        }

        Ok(doc)
    }
}