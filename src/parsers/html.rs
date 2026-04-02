//! HTML parser

use anyhow::Result;
use std::path::Path;
use scraper::{Html, Selector};

use crate::tree::{TreeNode, Document};
use super::Parser;

/// HTML parser
pub struct HtmlParser {
    name: String,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self {
            name: "html".to_string(),
        }
    }

    fn extract_text(&self, html: &Html, selector: &str) -> String {
        if let Ok(sel) = Selector::parse(selector) {
            html.select(&sel)
                .map(|el| el.text().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        }
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for HtmlParser {
    fn name(&self) -> &str {
        &self.name
    }

    fn extensions(&self) -> &[&str] {
        &["html", "htm"]
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

        let mut doc = Document::new(doc_id.clone(), doc_name.clone(), "html", path.to_path_buf());

        let html = Html::parse_document(content);
        let mut node_counter = 0;

        // Extract title
        let title = self.extract_text(&html, "title");
        doc.root.title = if title.is_empty() {
            doc_name.clone()
        } else {
            title.clone()
        };

        // Extract headings and create tree structure
        if let Ok(sel) = Selector::parse("h1, h2, h3, h4, h5, h6") {
            let mut stack: Vec<(usize, TreeNode)> = vec![];

            for element in html.select(&sel) {
                let tag = element.value().name();
                let level = tag[1..].parse::<usize>().unwrap_or(1);
                let text = element.text().collect::<String>();

                node_counter += 1;
                let node_id = format!("{}:heading:{}", doc_id, node_counter);
                let node = TreeNode::new(node_id, text);

                // Pop nodes with higher or equal level
                while stack.last().map(|(l, _)| *l >= level).unwrap_or(false) {
                    let (_, child) = stack.pop().unwrap();
                    if let Some((_, parent)) = stack.last_mut() {
                        parent.children.push(child);
                    } else {
                        doc.root.children.push(child);
                    }
                }

                stack.push((level, node));
            }

            // Pop remaining nodes
            while let Some((_, node)) = stack.pop() {
                if let Some((_, parent)) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    doc.root.children.push(node);
                }
            }
        }

        // Extract main content
        let main_content = self.extract_text(&html, "main, article, .content, #content");
        if !main_content.is_empty() {
            doc.root.text = main_content;
        } else {
            // Fallback to body text
            doc.root.text = self.extract_text(&html, "body");
        }

        // Extract code blocks
        if let Ok(sel) = Selector::parse("pre code, code") {
            let code: Vec<String> = html
                .select(&sel)
                .map(|el| el.text().collect::<String>())
                .collect();

            if !code.is_empty() {
                doc.root.code = Some(code.join("\n\n"));
            }
        }

        Ok(doc)
    }
}