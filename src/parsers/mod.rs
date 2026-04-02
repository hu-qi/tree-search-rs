//! Parser registry and common traits

mod markdown;
mod text;
mod code;
mod json;
mod csv;
mod xml;
mod html;

pub use markdown::MarkdownParser;
pub use text::TextParser;
pub use code::CodeParser;
pub use json::JsonParser;
pub use csv::CsvParser;
pub use xml::XmlParser;
pub use html::HtmlParser;

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use crate::tree::Document;
use crate::pathutil::FileType;

/// Parser trait for document parsing
pub trait Parser: Send + Sync {
    /// Parser name
    fn name(&self) -> &str;

    /// Supported file extensions
    fn extensions(&self) -> &[&str];

    /// Whether this parser produces tree-structured output
    fn tree_benefit(&self) -> bool;

    /// Parse content into a document
    fn parse(&self, content: &str, path: &Path) -> Result<Document>;
}

/// Registry of parsers
pub struct ParserRegistry {
    parsers: Vec<Arc<dyn Parser>>,
}

impl ParserRegistry {
    /// Create a new parser registry with default parsers
    pub fn new() -> Self {
        let parsers: Vec<Arc<dyn Parser>> = vec![
            Arc::new(MarkdownParser::new()),
            Arc::new(TextParser::new()),
            Arc::new(CodeParser::new()),
            Arc::new(JsonParser::new()),
            Arc::new(CsvParser::new()),
            Arc::new(XmlParser::new()),
            Arc::new(HtmlParser::new()),
        ];

        Self { parsers }
    }

    /// Get a parser for a file type
    pub fn get_parser(&self, ft: FileType) -> Option<&dyn Parser> {
        for parser in &self.parsers {
            if parser.extensions().iter().any(|ext| {
                ft.extensions().contains(&ext.as_str())
            }) {
                return Some(parser.as_ref());
            }
        }
        None
    }

    /// Register a custom parser
    pub fn register(&mut self, parser: Arc<dyn Parser>) {
        self.parsers.push(parser);
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}