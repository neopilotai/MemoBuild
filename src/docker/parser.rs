#[derive(Debug, Clone)]
pub enum Instruction {
    From(String),
    Workdir(String),
    Copy(String, String),
    Run(String),
    Env(String, String),
    Cmd(String),
    Git(String, String), // (url, target_dir)
    Other(String),
}

pub fn parse_dockerfile(content: &str) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }
        
        let keyword = parts[0].to_uppercase();
        let args = if line.len() > keyword.len() {
            line[keyword.len()..].trim()
        } else {
            ""
        };

        match keyword.as_str() {
            "FROM" => {
                if parts.len() >= 2 {
                    instructions.push(Instruction::From(parts[1].to_string()));
                }
            }
            "WORKDIR" => {
                if parts.len() >= 2 {
                    instructions.push(Instruction::Workdir(parts[1].to_string()));
                }
            }
            "COPY" => {
                if parts.len() >= 3 {
                    instructions.push(Instruction::Copy(
                        parts[1].to_string(),
                        parts[2].to_string(),
                    ));
                }
            }
            "RUN" => {
                instructions.push(Instruction::Run(args.to_string()));
            }
            "ENV" => {
                let env_parts: Vec<&str> = args.splitn(2, |c| c == ' ' || c == '=').collect();
                if env_parts.len() == 2 {
                    instructions.push(Instruction::Env(env_parts[0].to_string(), env_parts[1].to_string()));
                }
            }
            "CMD" => {
                instructions.push(Instruction::Cmd(args.to_string()));
            }
            "GIT" => {
                if parts.len() >= 3 {
                    instructions.push(Instruction::Git(
                        parts[1].to_string(),
                        parts[2].to_string(),
                    ));
                } else if parts.len() == 2 {
                    // Default target dir to the repo name or "."
                    instructions.push(Instruction::Git(
                        parts[1].to_string(),
                        ".".to_string(),
                    ));
                }
            }
            _ => {
                instructions.push(Instruction::Other(line.to_string()));
            }
        }
    }

    instructions
}
