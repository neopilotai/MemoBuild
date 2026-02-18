use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub content: String,
    pub hash: String,
    pub dirty: bool,
    pub deps: Vec<usize>,
    /// Set for COPY nodes — the source path to hash from the filesystem
    pub source_path: Option<PathBuf>,
    pub env: std::collections::HashMap<String, String>,
    pub cache_hit: bool,
}

#[derive(Debug)]
pub struct BuildGraph {
    pub nodes: Vec<Node>,
}

impl BuildGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Get nodes in topological order for execution
    pub fn topological_order(&self) -> Vec<usize> {
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = Vec::new();

        for i in 0..self.nodes.len() {
            if !visited[i] {
                self.dfs_topo(i, &mut visited, &mut stack);
            }
        }

        stack.reverse();
        stack
    }

    fn dfs_topo(&self, node: usize, visited: &mut Vec<bool>, stack: &mut Vec<usize>) {
        visited[node] = true;

        for &dep in &self.nodes[node].deps {
            if dep < self.nodes.len() && !visited[dep] {
                self.dfs_topo(dep, visited, stack);
            }
        }

        stack.push(node);
    }
}
