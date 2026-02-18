use anyhow::Result;
use crate::graph::Node;

#[derive(Debug, Clone)]
pub struct SandboxEnv {
    pub workspace_dir: std::path::PathBuf,
    pub env_vars: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub trait Sandbox: Send + Sync {
    fn prepare(&self, node: &Node) -> Result<SandboxEnv>;
    fn execute(&self, env: &SandboxEnv, node: &Node) -> Result<ExecResult>;
    fn cleanup(&self, env: &SandboxEnv) -> Result<()>;
}

pub mod local;
