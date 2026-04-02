//! Tests for tree module

use tree_search_rs::tree::{TreeNode, Document};
use std::path::PathBuf;

#[test]
fn test_tree_node_creation() {
    let node = TreeNode::new("test-id", "Test Title");
    assert_eq!(node.node_id, "test-id");
    assert_eq!(node.title, "Test Title");
    assert!(node.children.is_empty());
}

#[test]
fn test_tree_node_depth() {
    let mut root = TreeNode::new("root", "Root");
    assert_eq!(root.depth(), 0);

    root.children.push(TreeNode::new("child1", "Child 1"));
    assert_eq!(root.depth(), 1);

    root.children[0].children.push(TreeNode::new("grandchild", "Grandchild"));
    assert_eq!(root.depth(), 2);
}

#[test]
fn test_tree_node_count() {
    let mut root = TreeNode::new("root", "Root");
    root.children.push(TreeNode::new("child1", "Child 1"));
    root.children.push(TreeNode::new("child2", "Child 2"));
    root.children[0].children.push(TreeNode::new("grandchild", "Grandchild"));

    assert_eq!(root.count_nodes(), 4);
}

#[test]
fn test_tree_node_find() {
    let mut root = TreeNode::new("root", "Root");
    root.children.push(TreeNode::new("child1", "Child 1"));
    root.children.push(TreeNode::new("child2", "Child 2"));

    assert!(root.find_by_id("root").is_some());
    assert!(root.find_by_id("child1").is_some());
    assert!(root.find_by_id("child2").is_some());
    assert!(root.find_by_id("nonexistent").is_none());
}

#[test]
fn test_document_creation() {
    let doc = Document::new(
        "doc-1",
        "Test Document",
        "markdown",
        PathBuf::from("/path/to/doc.md"),
    );

    assert_eq!(doc.doc_id, "doc-1");
    assert_eq!(doc.doc_name, "Test Document");
    assert_eq!(doc.source_type, "markdown");
    assert_eq!(doc.tree_depth(), 0);
    assert!(!doc.tree_benefit());
}

#[test]
fn test_document_tree_benefit() {
    let mut doc = Document::new(
        "doc-1",
        "Test Document",
        "markdown",
        PathBuf::from("/path/to/doc.md"),
    );

    // Add children to root
    doc.root.children.push(TreeNode::new("child1", "Child 1"));
    doc.root.children[0].children.push(TreeNode::new("grandchild", "Grandchild"));

    assert!(doc.tree_benefit());
}