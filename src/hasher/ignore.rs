use std::path::Path;
use glob::Pattern;

/// Parsed ignore rules from .dockerignore or .gitignore
pub struct IgnoreRules {
    patterns: Vec<Pattern>,
}

impl IgnoreRules {
    pub fn empty() -> Self {
        Self { patterns: Vec::new() }
    }

    /// Load rules from a file (e.g. .dockerignore)
    pub fn from_file(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::empty(),
        };
        Self::from_str(&content)
    }

    /// Parse rules from a string using the glob crate for reliability.
    pub fn from_str(content: &str) -> Self {
        let patterns = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter_map(|l| Pattern::new(l).ok())
            .collect();
        Self { patterns }
    }

    /// Returns true if the given path (relative to the build context root) should be ignored
    pub fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.patterns {
            if pattern.matches(&path_str) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let rules = IgnoreRules::from_str("node_modules\n.git");
        assert!(rules.is_ignored(Path::new("node_modules")));
        assert!(rules.is_ignored(Path::new(".git")));
        assert!(!rules.is_ignored(Path::new("src")));
    }

    #[test]
    fn test_wildcard() {
        let rules = IgnoreRules::from_str("*.log");
        assert!(rules.is_ignored(Path::new("build.log")));
        assert!(!rules.is_ignored(Path::new("main.rs")));
    }
}
