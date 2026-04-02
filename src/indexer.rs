//! Document indexing pipeline

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::tree::Document;
use crate::parsers::{ParserRegistry, Parser};
use crate::fts::FtsIndex;
use crate::config::Config;

/// Indexer for building and maintaining the search index
pub struct Indexer {
    registry: ParserRegistry,
    config: Config,
    fts: Option<FtsIndex>,
}

impl Indexer {
    /// Create a new indexer with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            registry: ParserRegistry::new(),
            config,
            fts: None,
        }
    }

    /// Initialize the FTS index
    pub fn init_index(&mut self, path: &Path) -> Result<()> {
        self.fts = Some(FtsIndex::create(path)?);
        Ok(())
    }

    /// Index a single file
    pub fn index_file(&mut self, path: &Path) -> Result<Option<Document>> {
        // Detect file type
        let ft = crate::pathutil::FileType::from_path(path);

        // Get appropriate parser
        let parser = self.registry.get_parser(ft);
        if parser.is_none() {
            return Ok(None);
        }

        // Read file content
        let content = std::fs::read_to_string(path)?;

        // Parse document
        let parser = parser.unwrap();
        let doc = parser.parse(&content, path)?;

        // Add to FTS index
        if let Some(fts) = &mut self.fts {
            self.index_document(fts, &doc)?;
        }

        Ok(Some(doc))
    }

    /// Index a document in the FTS
    fn index_document(&self, fts: &mut FtsIndex, doc: &Document) -> Result<()> {
        for node in doc.root.iter_dfs() {
            fts.add_document(
                &node.node_id,
                &doc.doc_id,
                &node.title,
                node.summary.as_deref(),
                &node.text,
                node.code.as_deref(),
                node.front_matter.as_deref(),
            )?;
        }
        Ok(())
    }

    /// Index multiple files
    pub fn index_files(&mut self, paths: &[&Path]) -> Result<Vec<Document>> {
        let mut documents = Vec::new();

        for path in paths {
            if let Some(doc) = self.index_file(path)? {
                documents.push(doc);
            }
        }

        Ok(documents)
    }

    /// Commit the index
    pub fn commit(&mut self) -> Result<()> {
        if let Some(fts) = &mut self.fts {
            fts.commit()?;
            fts.reload()?;
        }
        Ok(())
    }

    /// Get the FTS index
    pub fn fts(&self) -> Option<&FtsIndex> {
        self.fts.as_ref()
    }
}

/// Build index for a set of paths
pub fn build_index(paths: &[&Path], config: &Config) -> Result<(Vec<Document>, FtsIndex)> {
    let mut indexer = Indexer::new(config.clone());
    indexer.init_index(&config.db_path)?;

    let documents = indexer.index_files(paths)?;
    indexer.commit()?;

    let fts = indexer.fts.unwrap();

    Ok((documents, fts))
}