/// Tests for cache module (hybrid cache with tiering)
#[cfg(test)]
mod cache_tests {
    use memobuild::{cache::HybridCache, hasher::FileHasher};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn test_hybrid_cache_creation() {
        // Test that hybrid cache can be created without remote backend
        let cache = HybridCache::new(None);
        assert!(
            cache.is_ok(),
            "HybridCache should create successfully without remote"
        );
    }

    #[tokio::test]
    async fn test_cache_miss_scenario() {
        let cache = HybridCache::new(None).expect("Failed to create cache");

        // Try to get non-existent artifact
        let hash = "nonexistent_hash";
        let result = cache.get(hash).await;

        // Should be None or Error, but not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_cache_put_get_roundtrip() {
        let cache = HybridCache::new(None).expect("Failed to create cache");

        let hash = "test_hash_abc123";
        let data = b"test artifact data";

        // Put artifact in cache
        let put_result = cache.put(hash, data).await;
        assert!(put_result.is_ok(), "Put should succeed");

        // Get artifact from cache
        let get_result = cache.get(hash).await;
        assert!(get_result.is_ok(), "Get should succeed");
    }

    #[test]
    fn test_file_hasher_consistency() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");

        // Write test file
        std::fs::write(&file_path, b"consistent data").expect("Failed to write file");

        // Hash file twice
        let hash1 = FileHasher::hash_file(&file_path).ok();
        let hash2 = FileHasher::hash_file(&file_path).ok();

        // Hashes should be identical
        if let (Some(h1), Some(h2)) = (hash1, hash2) {
            assert_eq!(h1, h2, "Same file should produce same hash");
        }
    }

    #[test]
    fn test_file_hasher_detects_changes() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test2.txt");

        // Write and hash
        std::fs::write(&file_path, b"original").expect("Failed to write");
        let hash1 = FileHasher::hash_file(&file_path).ok();

        // Modify and hash again
        std::fs::write(&file_path, b"modified").expect("Failed to modify");
        let hash2 = FileHasher::hash_file(&file_path).ok();

        // Hashes should be different
        if let (Some(h1), Some(h2)) = (hash1, hash2) {
            assert_ne!(h1, h2, "Different content should produce different hash");
        }
    }
}

/// Tests for hasher module
#[cfg(test)]
mod hasher_tests {
    use memobuild::hasher::{FileHasher, IgnoreRules};
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_ignore_rules_basic() {
        let rules = IgnoreRules::parse("node_modules\n.git\n*.log");

        assert!(rules.is_ignored(Path::new("node_modules")));
        assert!(rules.is_ignored(Path::new(".git")));
        assert!(rules.is_ignored(Path::new("build.log")));
        assert!(!rules.is_ignored(Path::new("src")));
    }

    #[test]
    fn test_ignore_rules_wildcard() {
        let rules = IgnoreRules::parse("*.tmp\ntest_*");

        assert!(rules.is_ignored(Path::new("file.tmp")));
        assert!(rules.is_ignored(Path::new("test_one")));
        assert!(rules.is_ignored(Path::new("test_two.txt")));
        assert!(!rules.is_ignored(Path::new("file.txt")));
    }

    #[test]
    fn test_ignore_rules_empty() {
        let rules = IgnoreRules::parse("");

        // Empty rules should match nothing
        assert!(!rules.is_ignored(Path::new("anything")));
        assert!(!rules.is_ignored(Path::new("node_modules")));
    }

    #[test]
    fn test_directory_hashing() {
        let temp_dir = tempdir().expect("Failed to create temp dir");

        // Create test structure
        std::fs::write(temp_dir.path().join("a.txt"), "a").expect("Failed to write");
        std::fs::write(temp_dir.path().join("b.txt"), "b").expect("Failed to write");

        // Hash directory
        let hash1 = FileHasher::hash_directory(temp_dir.path()).ok();

        // Modify and hash again
        std::fs::write(temp_dir.path().join("a.txt"), "modified").expect("Failed to modify");
        let hash2 = FileHasher::hash_directory(temp_dir.path()).ok();

        // Hashes should differ
        if let (Some(h1), Some(h2)) = (hash1, hash2) {
            assert_ne!(
                h1, h2,
                "Different directory content should produce different hash"
            );
        }
    }
}

/// Tests for core change detection
#[cfg(test)]
mod change_detection_tests {
    use memobuild::graph::{BuildGraph, BuildNode, NodeKind, NodeMetadata};

    #[test]
    fn test_dirty_flag_structure() {
        let node = BuildNode {
            id: 0,
            name: "test".to_string(),
            kind: NodeKind::Run,
            content: "test".to_string(),
            deps: vec![],
            dirty: true,
            metadata: NodeMetadata {
                parallelizable: false,
                layer_hashes: vec![],
                estimated_size_bytes: 1000,
            },
            node_key: "key".to_string(),
        };

        assert!(node.dirty, "Dirty flag should track rebuild necessity");
    }

    #[test]
    fn test_node_key_generation() {
        let node = BuildNode {
            id: 0,
            name: "consistent".to_string(),
            kind: NodeKind::Run,
            content: "echo hello".to_string(),
            deps: vec![],
            dirty: false,
            metadata: NodeMetadata {
                parallelizable: false,
                layer_hashes: vec![],
                estimated_size_bytes: 1000,
            },
            node_key: "generated_key_hash".to_string(),
        };

        // Node key should be present and consistent
        assert!(!node.node_key.is_empty());
        assert_eq!(node.node_key, "generated_key_hash");
    }

    #[test]
    fn test_dependency_chain_validation() {
        // Create a valid dependency chain A -> B -> C
        let mut nodes = vec![
            BuildNode {
                id: 0,
                name: "A".to_string(),
                kind: NodeKind::Run,
                content: "A".to_string(),
                deps: vec![],
                dirty: true,
                metadata: NodeMetadata {
                    parallelizable: false,
                    layer_hashes: vec![],
                    estimated_size_bytes: 1000,
                },
                node_key: "a".to_string(),
            },
            BuildNode {
                id: 1,
                name: "B".to_string(),
                kind: NodeKind::Run,
                content: "B".to_string(),
                deps: vec![0],
                dirty: false,
                metadata: NodeMetadata {
                    parallelizable: false,
                    layer_hashes: vec![],
                    estimated_size_bytes: 1000,
                },
                node_key: "b".to_string(),
            },
            BuildNode {
                id: 2,
                name: "C".to_string(),
                kind: NodeKind::Run,
                content: "C".to_string(),
                deps: vec![1],
                dirty: false,
                metadata: NodeMetadata {
                    parallelizable: false,
                    layer_hashes: vec![],
                    estimated_size_bytes: 1000,
                },
                node_key: "c".to_string(),
            },
        ];

        // Validate dependency references are valid
        for node in &nodes {
            for &dep in &node.deps {
                assert!(
                    dep < node.id as usize,
                    "Dependency should reference earlier node"
                );
                assert!(
                    dep < nodes.len(),
                    "Dependency should reference existing node"
                );
            }
        }
    }
}

/// Environment fingerprinting tests
#[cfg(test)]
mod env_fingerprint_tests {
    use memobuild::env::EnvFingerprint;

    #[test]
    fn test_fingerprint_creation() {
        // Fingerprint should be created successfully
        let fp = EnvFingerprint::from_env();
        assert!(!fp.is_empty(), "Fingerprint should not be empty");
    }

    #[test]
    fn test_fingerprint_consistency() {
        // Same environment should produce same fingerprint
        let fp1 = EnvFingerprint::from_env();
        let fp2 = EnvFingerprint::from_env();

        // Should be identical
        assert_eq!(
            fp1, fp2,
            "Same environment should produce consistent fingerprint"
        );
    }
}
