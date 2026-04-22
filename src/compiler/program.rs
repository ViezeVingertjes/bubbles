//! [`Program`] — the compiled output of one or more `.bub` sources.

use indexmap::IndexMap;

use crate::compiler::ast::Node;
use crate::error::{DialogueError, Result};

/// A compiled dialogue program ready to be executed by the runner.
#[derive(Debug, Clone)]
pub struct Program {
    /// Nodes keyed by title. Multiple nodes with the same title form a node group.
    pub(crate) nodes: IndexMap<String, Vec<Node>>,
}

impl Program {
    /// Returns `true` if the program contains a node with the given title.
    #[must_use]
    pub fn node_exists(&self, title: &str) -> bool {
        self.nodes.contains_key(title)
    }

    /// Iterates over all node titles in insertion order.
    pub fn node_titles(&self) -> impl Iterator<Item = &str> {
        self.nodes.keys().map(String::as_str)
    }

    /// Returns the tags of the first node with the given title, if any.
    #[must_use]
    pub fn node_tags(&self, title: &str) -> Option<&[String]> {
        self.nodes.get(title)?.first().map(|n| n.tags.as_slice())
    }

    pub(crate) fn node_group(&self, title: &str) -> Option<&[Node]> {
        self.nodes.get(title).map(Vec::as_slice)
    }

    pub(crate) fn from_nodes(nodes: Vec<Node>) -> Result<Self> {
        let mut map: IndexMap<String, Vec<Node>> = IndexMap::new();
        for node in nodes {
            let entry = map.entry(node.title.clone()).or_default();
            // Duplicate non-grouped nodes are an error
            if !entry.is_empty() {
                let existing_ungrouped = entry.iter().all(|n| n.when_src.is_none());
                if existing_ungrouped && node.when_src.is_none() {
                    return Err(DialogueError::DuplicateNode(node.title));
                }
            }
            entry.push(node);
        }
        Ok(Self { nodes: map })
    }
}
