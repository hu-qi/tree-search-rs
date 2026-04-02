# TreeSearch Rust

A tree-aware document search engine built with Rust and Tantivy FTS.

## Features

- **Tree-aware search**: Documents with hierarchical structure (Markdown, JSON, XML, HTML) benefit from tree-based search algorithms
- **Multiple parsers**: Support for Markdown, text, code (Python, JS, Java, Go, Rust, C++), JSON, CSV, XML, HTML
- **Full-text search**: Powered by Tantivy with Chinese (jieba) and English tokenization
- **Multiple search modes**: Auto, Flat, Tree
- **CLI interface**: Simple command-line tool

## Installation

```bash
cargo install tree-search-rs
```

## Usage

### Quick Search

```bash
# Search with auto-built index
treesearch search "query" path/

# Specify search mode
treesearch search "query" path/ --mode tree

# Limit results
treesearch search "query" path/ -n 20
```

### Build Index

```bash
# Index specific paths
treesearch index src/ docs/

# With options
treesearch index src/ docs/ --max-files 50000 --max-nodes 2000
```

### Document Info

```bash
treesearch info document.md
```

## Search Modes

| Mode | Description |
|------|-------------|
| `auto` | Automatically select based on document characteristics |
| `flat` | Direct FTS scoring, no tree traversal |
| `tree` | FTS anchor + best-first tree walk + path aggregation |

## Supported File Types

| Type | Extensions | Tree-Aware |
|------|------------|------------|
| Markdown | `.md` | ✅ |
| Text | `.txt` | ❌ |
| Python | `.py` | ❌ |
| JavaScript | `.js`, `.jsx`, `.ts`, `.tsx` | ❌ |
| Java | `.java` | ❌ |
| Go | `.go` | ❌ |
| Rust | `.rs` | ❌ |
| C/C++ | `.c`, `.cpp`, `.h`, `.hpp` | ❌ |
| JSON | `.json`, `.jsonl` | ✅ |
| CSV | `.csv` | ❌ |
| XML | `.xml` | ✅ |
| HTML | `.html`, `.htm` | ✅ |

## Architecture

```
tree-search-rs/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root
│   ├── tree.rs          # TreeNode, Document structures
│   ├── indexer.rs       # Indexing pipeline
│   ├── search.rs        # Search engine
│   ├── fts.rs           # Tantivy FTS wrapper
│   ├── tokenizer.rs     # CJK + English tokenization
│   ├── config.rs        # Configuration
│   ├── pathutil.rs      # Path utilities
│   ├── heuristics.rs    # Scoring heuristics
│   └── parsers/         # Document parsers
│       ├── markdown.rs
│       ├── text.rs
│       ├── code.rs
│       ├── json.rs
│       ├── csv.rs
│       ├── xml.rs
│       └── html.rs
```

## License

MIT