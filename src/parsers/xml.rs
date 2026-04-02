//! XML parser

use anyhow::Result;
use std::path::Path;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::tree::{TreeNode, Document};
use super::Parser;

/// XML parser
pub struct XmlParser {
    name: String,
}

impl XmlParser {
    pub fn new() -> Self {
        Self {
            name: "xml".to_string(),
        }
    }
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for XmlParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
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

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "xml", path.to_path_buf());

        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut node_stack: Vec<TreeNode> = vec![TreeNode::new(format!("{}:root", doc_id), doc_name.clone())];
        let mut node_counter = 0;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    node_counter += 1;
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let node_id = format!("{}:elem:{}", doc_id, node_counter);

                    let node = TreeNode::new(node_id, tag);
                    node_stack.push(node);
                }
                Ok(Event::End(_)) => {
                    if node_stack.len() > 1 {
                        let node = node_stack.pop().unwrap();
                        node_stack.last_mut().unwrap().children.push(node);
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.to_string();
                    if !text.trim().is_empty() {
                        if let Some(node) = node_stack.last_mut() {
                            if node.text.is_empty() {
                                node.text = text;
                            } else {
                                node.text.push(' ');
                                node.text.push_str(&text);
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    anyhow::bail!("Error parsing XML: {:?}", e);
                }
                _ => {}
            }
            buf.clear();
        }

        // Pop remaining nodes
        while node_stack.len() > 1 {
            let node = node_stack.pop().unwrap();
            node_stack.last_mut().unwrap().children.push(node);
        }

        doc.root = node_stack.pop().unwrap();

        Ok(doc)
    }
}