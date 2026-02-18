use crate::docker::parser::Instruction;
use crate::graph::{BuildGraph, Node};
use std::path::PathBuf;

/// Convert a flat list of Dockerfile instructions into a dependency graph.
/// Each node depends on the previous one (linear chain).
///
/// Fix 3 — COPY "." case: when src is "." we set source_path to the
/// current working directory (project root) so the entire context is hashed.
pub fn build_graph_from_instructions(instructions: Vec<Instruction>) -> BuildGraph {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let mut nodes: Vec<Node> = Vec::new();
    let mut last_id: Option<usize> = None;

    for (i, instr) in instructions.iter().enumerate() {
        let name = format!("{:?}", instr);

        let mut env = std::collections::HashMap::new();

        let (content, source_path, kind) = match instr {
            Instruction::From(img) => (format!("FROM {}", img), None, crate::graph::NodeKind::From),
            Instruction::Workdir(dir) => (
                format!("WORKDIR {}", dir),
                None,
                crate::graph::NodeKind::Workdir,
            ),
            Instruction::Copy(src, dst) => {
                let path = if src == "." {
                    // Fix 3: COPY . . → hash entire project root
                    project_root.clone()
                } else {
                    project_root.join(src)
                };
                (
                    format!("COPY {} {}", src, dst),
                    Some(path),
                    crate::graph::NodeKind::Copy {
                        src: PathBuf::from(src),
                    },
                )
            }
            Instruction::Run(cmd) => (format!("RUN {}", cmd), None, crate::graph::NodeKind::Run),
            Instruction::Env(key, value) => {
                env.insert(key.clone(), value.clone());
                (
                    format!("ENV {}={}", key, value),
                    None,
                    crate::graph::NodeKind::Env,
                )
            }
            Instruction::Cmd(cmd) => (format!("CMD {}", cmd), None, crate::graph::NodeKind::Other),
            Instruction::Git(url, target) => (
                format!("GIT {} {}", url, target),
                None,
                crate::graph::NodeKind::Other,
            ),
            Instruction::Other(s) => (s.clone(), None, crate::graph::NodeKind::Other),
        };

        let node = Node {
            id: i,
            name,
            content,
            kind,
            hash: "".into(),
            dirty: false,
            source_path,
            env,
            cache_hit: false,
            deps: match last_id {
                Some(prev) => vec![prev],
                None => vec![],
            },
        };

        nodes.push(node);
        last_id = Some(i);
    }

    BuildGraph { nodes }
}
