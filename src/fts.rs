//! Full-text search using Tantivy

use anyhow::Result;
use std::path::Path;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::*,
    Index, IndexReader, IndexWriter, TantivyDocument,
};

/// FTS index wrapper
pub struct FtsIndex {
    schema: Schema,
    index: Index,
    reader: IndexReader,
    writer: Option<IndexWriter>,
}

/// Schema field identifiers
pub struct Fields {
    pub node_id: Field,
    pub doc_id: Field,
    pub title: Field,
    pub summary: Field,
    pub body: Field,
    pub code: Field,
    pub front_matter: Field,
}

impl FtsIndex {
    /// Create a new FTS index at the given path
    pub fn create(path: &Path) -> Result<Self> {
        let schema = Self::build_schema();
        let index = Index::create_in_dir(path, schema.clone())?;
        let reader = index.reader()?;
        Ok(Self {
            schema,
            index,
            reader,
            writer: None,
        })
    }

    /// Open an existing FTS index
    pub fn open(path: &Path) -> Result<Self> {
        let index = Index::open_in_dir(path)?;
        let schema = index.schema();
        let reader = index.reader()?;
        Ok(Self {
            schema,
            index,
            reader,
            writer: None,
        })
    }

    /// Create an in-memory index for testing
    pub fn create_in_memory() -> Result<Self> {
        let schema = Self::build_schema();
        let index = Index::create_in_ram(schema.clone());
        let reader = index.reader()?;
        Ok(Self {
            schema,
            index,
            reader,
            writer: None,
        })
    }

    /// Build the index schema
    fn build_schema() -> Schema {
        let mut builder = Schema::builder();

        builder.add_text_field("node_id", STRING | STORED);
        builder.add_text_field("doc_id", STRING | STORED);
        builder.add_text_field("title", TEXT | STORED);
        builder.add_text_field("summary", TEXT);
        builder.add_text_field("body", TEXT);
        builder.add_text_field("code", TEXT);
        builder.add_text_field("front_matter", TEXT);

        builder.build()
    }

    /// Get field references
    pub fn fields(&self) -> Fields {
        Fields {
            node_id: self.schema.get_field("node_id").unwrap(),
            doc_id: self.schema.get_field("doc_id").unwrap(),
            title: self.schema.get_field("title").unwrap(),
            summary: self.schema.get_field("summary").unwrap(),
            body: self.schema.get_field("body").unwrap(),
            code: self.schema.get_field("code").unwrap(),
            front_matter: self.schema.get_field("front_matter").unwrap(),
        }
    }

    /// Start a write transaction
    pub fn begin_write(&mut self) -> Result<()> {
        let writer = self.index.writer(50_000_000)?; // 50MB heap
        self.writer = Some(writer);
        Ok(())
    }

    /// Add a document to the index
    pub fn add_document(
        &mut self,
        node_id: &str,
        doc_id: &str,
        title: &str,
        summary: Option<&str>,
        body: &str,
        code: Option<&str>,
        front_matter: Option<&str>,
    ) -> Result<()> {
        let fields = self.fields();
        let writer = self.writer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No write transaction in progress"))?;

        let mut doc = TantivyDocument::new();

        doc.add_text(fields.node_id, node_id);
        doc.add_text(fields.doc_id, doc_id);
        doc.add_text(fields.title, title);
        if let Some(s) = summary {
            doc.add_text(fields.summary, s);
        }
        doc.add_text(fields.body, body);
        if let Some(c) = code {
            doc.add_text(fields.code, c);
        }
        if let Some(fm) = front_matter {
            doc.add_text(fields.front_matter, fm);
        }

        writer.add_document(doc)?;
        Ok(())
    }

    /// Commit the write transaction
    pub fn commit(&mut self) -> Result<()> {
        if let Some(mut writer) = self.writer.take() {
            writer.commit()?;
        }
        Ok(())
    }

    /// Search the index
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let searcher = self.reader.searcher();
        let fields = self.fields();

        let mut query_parser = QueryParser::for_index(&self.index, vec![
            fields.title,
            fields.summary,
            fields.body,
            fields.code,
        ]);

        // Boost title matches
        query_parser.set_field_boost(fields.title, 10.0);
        query_parser.set_field_boost(fields.summary, 5.0);
        query_parser.set_field_boost(fields.code, 2.0);

        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut hits = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            let node_id = doc.get_first(fields.node_id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let doc_id = doc.get_first(fields.doc_id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = doc.get_first(fields.title)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            hits.push(SearchHit {
                node_id,
                doc_id,
                title,
                score,
            });
        }

        Ok(hits)
    }

    /// Reload the reader to see new documents
    pub fn reload(&self) -> Result<()> {
        self.reader.reload()?;
        Ok(())
    }
}

/// A search hit from the FTS index
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub node_id: String,
    pub doc_id: String,
    pub title: String,
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory() {
        let index = FtsIndex::create_in_memory();
        assert!(index.is_ok());
    }

    #[test]
    fn test_add_and_search() -> Result<()> {
        let mut index = FtsIndex::create_in_memory()?;

        index.begin_write()?;
        index.add_document(
            "node1",
            "doc1",
            "Introduction to Rust",
            Some("A brief introduction"),
            "Rust is a systems programming language",
            Some("fn main() {}"),
            None,
        )?;
        index.commit()?;
        index.reload()?;

        let hits = index.search("Rust", 10)?;
        assert!(!hits.is_empty());

        Ok(())
    }
}