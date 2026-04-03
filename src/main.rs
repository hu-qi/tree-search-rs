//! TreeSearch CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod tree;
mod config;
mod search;
mod fts;
mod tokenizer;
mod indexer;
mod pathutil;
mod heuristics;
mod parsers;

use config::{Config, SearchModeConfig};
use search::SearchMode;

#[derive(Parser)]
#[command(name = "treesearch")]
#[command(about = "Tree-aware document search engine")]
#[command(version)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Quick search (auto-build index if needed)
    Search {
        /// Search query
        query: String,
        /// Path to search
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Search mode (auto, flat, tree)
        #[arg(short, long, default_value = "auto")]
        mode: SearchModeConfig,
        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
        /// Maximum results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },
    /// Build index
    Index {
        /// Paths to index
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Database path
        #[arg(short, long)]
        db: Option<PathBuf>,
        /// Maximum nodes per document
        #[arg(long, default_value = "1000")]
        max_nodes: usize,
        /// Maximum files to index
        #[arg(long, default_value = "10000")]
        max_files: usize,
    },
    /// Show document info
    Info {
        /// Path to document
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbose flag
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    match cli.command {
        Commands::Search { query, path, mode, db, limit } => {
            search_cmd(query, path, mode, db, limit, cli.verbose)?;
        }
        Commands::Index { paths, db, max_nodes, max_files } => {
            index_cmd(paths, db, max_nodes, max_files, cli.verbose)?;
        }
        Commands::Info { path } => {
            info_cmd(path, cli.verbose)?;
        }
    }

    Ok(())
}

fn search_cmd(
    query: String,
    path: PathBuf,
    mode: SearchModeConfig,
    db: Option<PathBuf>,
    limit: usize,
    verbose: bool,
) -> Result<()> {
    use fts::FtsIndex;
    use search::SearchEngine;

    println!("Searching for: {}", query);
    if verbose {
        println!("Path: {:?}", path);
        println!("Mode: {}", mode);
        println!("Limit: {}", limit);
    }

    // Determine search mode
    let search_mode = match mode {
        SearchModeConfig::Auto => SearchMode::Auto,
        SearchModeConfig::Flat => SearchMode::Flat,
        SearchModeConfig::Tree => SearchMode::Tree,
    };

    // Check if index exists
    let db_path = db.unwrap_or_else(|| path.join(".treesearch.db"));

    if verbose {
        println!("Database path: {:?}", db_path);
    }

    if db_path.exists() {
        // Use existing index
        let index = FtsIndex::open(&db_path)?;
        
        if verbose {
            println!("Using existing index at {:?}", db_path);
        }
        
        let hits = index.search(&query, limit)?;

        if hits.is_empty() {
            println!("No results found.");
        } else {
            println!("\nFound {} results:\n", hits.len());
            for (i, hit) in hits.iter().enumerate() {
                println!("{}. {} (score: {:.3})", i + 1, hit.title, hit.score);
                println!("   Node: {}", hit.node_id);
                if verbose {
                    println!("   Document: {}", hit.doc_id);
                }
                println!();
            }
        }
    } else {
        // No index, perform quick search (parse and search on-the-fly)
        use pathutil::{discover_files, DiscoveryOptions};
        use parsers::ParserRegistry;
        use tree::Document;

        println!("No index found. Performing quick search (may be slow)...");

        // Discover files
        let options = DiscoveryOptions::default();
        let files = discover_files(&path, &options)?;

        if files.is_empty() {
            println!("No files found in path.");
            return Ok(());
        }

        if verbose {
            println!("Discovered {} files", files.len());
        }

        println!("Found {} files, searching...", files.len());

        // Parse documents
        let registry = ParserRegistry::new();
        let mut documents: Vec<Document> = Vec::new();

        for file in &files {
            let ft = pathutil::FileType::from_path(file);
            if let Some(parser) = registry.get_parser(ft) {
                if let Ok(content) = std::fs::read_to_string(file) {
                    if let Ok(doc) = parser.parse(&content, file) {
                        documents.push(doc);
                        tracing::debug!("Parsed: {:?}", file);
                    }
                }
            }
        }

        println!("Parsed {} documents", documents.len());

        // Create in-memory index
        let mut index = FtsIndex::create_in_memory()?;
        index.begin_write()?;

        if verbose {
            println!("Building in-memory index...");
        }

        for doc in &documents {
            for node in doc.root.iter_dfs() {
                index.add_document(
                    &node.node_id,
                    &doc.doc_id,
                    &node.title,
                    node.summary.as_deref(),
                    &node.text,
                    node.code.as_deref(),
                    node.front_matter.as_deref(),
                )?;
            }
        }

        index.commit()?;
        index.reload()?;

        // Search
        let hits = index.search(&query, limit)?;

        if hits.is_empty() {
            println!("No results found.");
        } else {
            println!("\nFound {} results:\n", hits.len());
            for (i, hit) in hits.iter().enumerate() {
                println!("{}. {} (score: {:.3})", i + 1, hit.title, hit.score);
                println!("   Node: {}", hit.node_id);
                if verbose {
                    println!("   Document: {}", hit.doc_id);
                }
                println!();
            }
        }
    }

    Ok(())
}

fn index_cmd(
    paths: Vec<PathBuf>,
    db: Option<PathBuf>,
    max_nodes: usize,
    max_files: usize,
    verbose: bool,
) -> Result<()> {
    use indexer::Indexer;
    use pathutil::{discover_files, DiscoveryOptions};

    println!("Building index...");
    if verbose {
        println!("Paths: {:?}", paths);
        println!("Max nodes per doc: {}", max_nodes);
        println!("Max files: {}", max_files);
    }

    // Build config
    let mut config = Config::default();
    config.max_nodes_per_doc = max_nodes;
    config.max_files = max_files;

    if let Some(db_path) = db {
        config.db_path = db_path;
    } else if let Some(first_path) = paths.first() {
        config.db_path = first_path.join(".treesearch.db");
    }

    if verbose {
        println!("Database path: {:?}", config.db_path);
    }

    // Discover files
    let options = DiscoveryOptions::default();
    let mut all_files = Vec::new();

    for path in &paths {
        let files = discover_files(path, &options)?;
        all_files.extend(files);
    }

    println!("Found {} files", all_files.len());

    if all_files.len() > max_files {
        println!("Warning: Too many files, limiting to {}", max_files);
        all_files.truncate(max_files);
    }

    // Build index
    let mut indexer = Indexer::new(config.clone());
    indexer.init_index(&config.db_path)?;

    let mut count = 0;
    for file in &all_files {
        if let Some(doc) = indexer.index_file(file)? {
            count += 1;
            if verbose {
                println!("[{}/{}] Indexed: {:?}", count, all_files.len(), file);
            } else if count % 100 == 0 {
                println!("Indexed {} documents...", count);
            }
        }
    }

    indexer.commit()?;

    println!("\nIndex built successfully!");
    println!("Indexed {} documents", count);
    println!("Index location: {:?}", config.db_path);

    Ok(())
}

fn info_cmd(path: PathBuf, verbose: bool) -> Result<()> {
    use pathutil::FileType;
    use parsers::ParserRegistry;

    println!("File: {:?}", path);

    // Detect file type
    let ft = FileType::from_path(&path);
    println!("Type: {:?}", ft);
    println!("Tree benefit: {}", ft.tree_benefit());

    // Get parser
    let registry = ParserRegistry::new();
    if let Some(parser) = registry.get_parser(ft) {
        println!("Parser: {}", parser.name());

        // Parse document
        let content = std::fs::read_to_string(&path)?;
        let doc = parser.parse(&content, &path)?;

        println!("\nDocument info:");
        println!("  ID: {}", doc.doc_id);
        println!("  Name: {}", doc.doc_name);
        println!("  Source type: {}", doc.source_type);
        println!("  Tree depth: {}", doc.tree_depth());
        println!("  Node count: {}", doc.count_nodes());
        println!("  Tree benefit: {}", doc.tree_benefit());
        
        if verbose {
            println!("\nVerbose info:");
            println!("  File size: {} bytes", content.len());
            // Count nodes by type (computed from content fields)
            let mut node_types = std::collections::HashMap::new();
            for node in doc.root.iter_dfs() {
                let node_kind = if node.code.is_some() {
                    "code"
                } else if node.front_matter.is_some() {
                    "front_matter"
                } else if !node.text.is_empty() {
                    "text"
                } else {
                    "structural"
                };
                *node_types.entry(node_kind).or_insert(0) += 1;
            }
            println!("  Node types: {:?}", node_types);
        }
    } else {
        println!("No parser available for this file type.");
    }

    Ok(())
}