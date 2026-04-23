//! [`Program`] - the compiled output of one or more `.bub` sources.

use indexmap::IndexMap;

use crate::compiler::ast::{Node, Stmt};
use crate::error::{DialogueError, Result};

/// A `<<declare $var = expr>>` entry collected from a compiled program.
///
/// These declarations are useful for save systems that need to know the
/// full set of variables a script uses and their default expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDecl {
    /// Variable name including the `$` sigil.
    pub name: String,
    /// The default-value expression source, as written in the script.
    pub default_src: String,
}

/// A compiled dialogue program ready to be executed by the runner.
#[derive(Debug, Clone)]
pub struct Program {
    /// Nodes keyed by title. Multiple nodes with the same title form a node group.
    pub(crate) nodes: IndexMap<String, Vec<Node>>,
    /// All `<<declare>>` statements collected across every node, in encounter order.
    declarations: Vec<VariableDecl>,
}

impl Program {
    /// Returns `true` if the program contains a node with the given title.
    ///
    /// # Example
    ///
    /// ```rust
    /// let prog = bubbles::compile("title: Start\n---\n===\n").unwrap();
    /// assert!(prog.node_exists("Start"));
    /// assert!(!prog.node_exists("Missing"));
    /// ```
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

    /// Returns all `<<declare>>` variable declarations found in the program.
    ///
    /// This is useful for save systems that need to enumerate every variable
    /// a script declares, and for editor tooling.
    ///
    /// # Example
    ///
    /// ```rust
    /// let prog = bubbles::compile(
    ///     "title: Start\n---\n<<declare $health = 100>>\n===\n"
    /// ).unwrap();
    /// let decls = prog.variable_declarations();
    /// assert_eq!(decls.len(), 1);
    /// assert_eq!(decls[0].name, "$health");
    /// assert_eq!(decls[0].default_src, "100");
    /// ```
    #[must_use]
    pub fn variable_declarations(&self) -> &[VariableDecl] {
        &self.declarations
    }

    pub(crate) fn node_group(&self, title: &str) -> Option<&[Node]> {
        self.nodes.get(title).map(Vec::as_slice)
    }

    pub(crate) fn from_nodes(nodes: Vec<Node>) -> Result<Self> {
        let mut map: IndexMap<String, Vec<Node>> = IndexMap::new();
        let mut declarations: Vec<VariableDecl> = Vec::new();
        let mut seen_decls: indexmap::IndexSet<String> = indexmap::IndexSet::new();

        for node in nodes {
            collect_declarations(&node.body, &mut declarations, &mut seen_decls);
            let entry = map.entry(node.title.clone()).or_default();
            // Duplicate non-grouped nodes are an error.
            if !entry.is_empty() {
                let existing_ungrouped = entry.iter().all(|n| n.when.is_none());
                if existing_ungrouped && node.when.is_none() {
                    return Err(DialogueError::DuplicateNode(node.title));
                }
            }
            entry.push(node);
        }
        Ok(Self {
            nodes: map,
            declarations,
        })
    }
}

/// Recursively collect `Stmt::Declare` entries from a body, deduplicating by name.
fn collect_declarations(
    stmts: &[Stmt],
    out: &mut Vec<VariableDecl>,
    seen: &mut indexmap::IndexSet<String>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Declare {
                name, default_src, ..
            } if seen.insert(name.clone()) => {
                out.push(VariableDecl {
                    name: name.clone(),
                    default_src: default_src.clone(),
                });
            }
            Stmt::If {
                branches,
                else_body,
            } => {
                for b in branches {
                    collect_declarations(&b.body, out, seen);
                }
                collect_declarations(else_body, out, seen);
            }
            Stmt::Once {
                body, else_body, ..
            } => {
                collect_declarations(body, out, seen);
                collect_declarations(else_body, out, seen);
            }
            Stmt::Options(items) => {
                for item in items {
                    collect_declarations(&item.body, out, seen);
                }
            }
            _ => {}
        }
    }
}
