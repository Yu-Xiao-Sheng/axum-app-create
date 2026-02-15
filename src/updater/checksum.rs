// SHA-256 checksum calculator
//
// This module provides SHA-256 checksum calculation for file content,
// used by the update engine to detect user modifications.

use crate::error::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// SHA-256 checksum calculator
pub struct ChecksumCalculator;

impl ChecksumCalculator {
    /// Calculate SHA-256 checksum of byte content
    ///
    /// Returns hex-encoded lowercase SHA-256 hash string
    pub fn calculate(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Calculate checksums for all specified files in a project directory
    ///
    /// # Arguments
    /// * `project_dir` - Root directory of the project
    /// * `files` - List of relative file paths to checksum
    ///
    /// # Returns
    /// HashMap mapping relative file path to its SHA-256 checksum
    pub fn calculate_all(project_dir: &Path, files: &[String]) -> Result<HashMap<String, String>> {
        let mut checksums = HashMap::new();
        for file in files {
            let file_path = project_dir.join(file);
            if file_path.exists() {
                let content = std::fs::read(&file_path)?;
                checksums.insert(file.clone(), Self::calculate(&content));
            }
        }
        Ok(checksums)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_known_hash() {
        // SHA-256 of empty string
        let hash = ChecksumCalculator::calculate(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_calculate_hello_world() {
        let hash = ChecksumCalculator::calculate(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_calculate_all_with_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("a.txt"), "hello").unwrap();
        std::fs::write(temp_dir.path().join("b.txt"), "world").unwrap();

        let files = vec!["a.txt".to_string(), "b.txt".to_string()];
        let checksums = ChecksumCalculator::calculate_all(temp_dir.path(), &files).unwrap();

        assert_eq!(checksums.len(), 2);
        assert_eq!(checksums["a.txt"], ChecksumCalculator::calculate(b"hello"));
        assert_eq!(checksums["b.txt"], ChecksumCalculator::calculate(b"world"));
    }

    #[test]
    fn test_calculate_all_skips_missing_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("exists.txt"), "data").unwrap();

        let files = vec!["exists.txt".to_string(), "missing.txt".to_string()];
        let checksums = ChecksumCalculator::calculate_all(temp_dir.path(), &files).unwrap();

        assert_eq!(checksums.len(), 1);
        assert!(checksums.contains_key("exists.txt"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 5: SHA-256 checksum determinism
        /// For any byte sequence, computing the checksum twice produces the same result.
        /// Feature: v030-template-and-update, Property 5: SHA-256 checksum determinism
        /// **Validates: Requirements 6.1, 6.2, 6.3**
        #[test]
        fn prop_checksum_determinism(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
            let hash1 = ChecksumCalculator::calculate(&data);
            let hash2 = ChecksumCalculator::calculate(&data);
            prop_assert_eq!(&hash1, &hash2, "Same input must produce same checksum");
        }

        /// Property 5 (collision resistance): Different inputs produce different checksums.
        /// Feature: v030-template-and-update, Property 5: SHA-256 checksum determinism
        /// **Validates: Requirements 6.1, 6.2, 6.3**
        #[test]
        fn prop_checksum_collision_resistance(
            data1 in proptest::collection::vec(any::<u8>(), 1..512),
            data2 in proptest::collection::vec(any::<u8>(), 1..512),
        ) {
            prop_assume!(data1 != data2);
            let hash1 = ChecksumCalculator::calculate(&data1);
            let hash2 = ChecksumCalculator::calculate(&data2);
            prop_assert_ne!(hash1, hash2, "Different inputs should produce different checksums");
        }
    }
}
