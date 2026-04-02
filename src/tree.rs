//! Core data structures for tree-based document representation

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// A node in the document tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// Unique node identifier
    pub node_id: String,
    /// Node title (e.g., section heading)
    pub title: String,
    /// Optional summary/description
    pub summary: Option<String>,
    /// Main text content
    pub text: String,
    /// Optional code block content
    pub code: Option<String>,
    /// Optional front matter (YAML/JSON)
    pub front_matter: Option<String>,
    /// Child nodes
    pub children: Vec<TreeNode>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TreeNode {
    /// Create a new tree node with the given title
    pub fn new(node_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            title: title.into(),
            summary: None,
            text: String::new(),
            code: None,
            front_matter: None,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Calculate the depth of this node (0 for leaf)
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Count total nodes in subtree (including self)
    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
    }

    /// Find a node by ID
    pub fn find_by_id(&self, node_id: &str) -> Option<&TreeNode> {
        if self.node_id == node_id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_id(node_id) {
                return Some(found);
            }
        }
        None
    }

    /// Collect all node IDs in subtree
    pub fn collect_node_ids(&self) -> Vec<String> {
        let mut ids = vec![self.node_id.clone()];
        for child in &self.children {
            ids.extend(child.collect_node_ids());
        }
        ids
    }

    /// Iterate over all nodes in DFS order
    pub fn iter_dfs(&self) -> TreeNodeIter {
        TreeNodeIter::new(self)
    }
}

/// DFS iterator for tree nodes
pub struct TreeNodeIter<'a> {
    stack: Vec<&'a TreeNode>,
}

impl<'a> TreeNodeIter<'a> {
    fn new(root: &'a TreeNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for TreeNodeIter<'a> {
    type Item = &'a TreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // Push children in reverse order for correct DFS order
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}

/// A document with tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document identifier
    pub doc_id: String,
    /// Document name/title
    pub doc_name: String,
    /// Source type (markdown, code, json, etc.)
    pub source_type: String,
    /// Root node of the document tree
    pub root: TreeNode,
    /// File path
    pub path: PathBuf,
}

impl Document {
    /// Create a new document
    pub fn new(
        doc_id: impl Into<String>,
        doc_name: impl Into<String>,
        source_type: impl Into<String>,
        path: PathBuf,
    ) -> Self {
        let doc_id = doc_id.into();
        let root = TreeNode::new(format!("{}:root", doc_id), doc_name.clone());
        Self {
            doc_id,
            doc_name,
            source_type: source_type.into(),
            root,
            path,
        }
    }

    /// Calculate tree depth
    pub fn tree_depth(&self) -> usize {
        self.root.depth()
    }

    /// Check if document benefits from tree-based search
    pub fn tree_benefit(&self) -> bool {
        self.tree_depth() >= 2
    }

    /// Count total nodes
    pub fn count_nodes(&self) -> usize {
        self.root.count_nodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(root.count_nodes(), 3);
    }

    #[test]
    fn test_dfs_iteration() {
        let mut root = TreeNode::new("root", "Root");
        root.children.push(TreeNode::new("child1", "Child 1"));
        root.children.push(TreeNode::new("child2", "Child 2"));

        let ids: Vec<_> = root.iter_dfs().map(|n| n.node_id.as_str()).collect();
        assert_eq!(ids, vec!["root", "child1", "child2"]);
    }
}