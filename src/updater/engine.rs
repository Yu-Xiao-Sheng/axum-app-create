// Update engine
//
// This module orchestrates the project update workflow:
// 1. Read .axum-app-create.json metadata
// 2. Regenerate templates using stored config
// 3. Compare files and classify (skip/auto-update/conflict)
// 4. Apply updates or report in dry-run mode

use crate::error::Result;
use crate::template::context::TemplateContext;
use crate::template::engine::TemplateEngine;
use crate::template::resolver::TemplateResolver;
use crate::updater::checksum::ChecksumCalculator;
use crate::updater::metadata::MetadataManager;
use std::collections::HashMap;
use std::path::PathBuf;

/// File classification during update
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileClassification {
    /// File content is identical to new generation — skip
    Skip,
    /// File differs but user hasn't modified it — safe to auto-update
    AutoUpdate,
    /// File differs and user has modified it — conflict
    Conflict,
    /// File is new (not in previous generation)
    New,
}

/// Classify a single file for update
///
/// This is a pure function for property testing.
pub fn classify_file(
    current_content: Option<&[u8]>,
    new_content: &[u8],
    stored_checksum: Option<&str>,
) -> FileClassification {
    match current_content {
        None => FileClassification::New,
        Some(current) => {
            if current == new_content {
                FileClassification::Skip
            } else {
                let current_checksum = ChecksumCalculator::calculate(current);
                match stored_checksum {
                    Some(stored) if current_checksum == stored => {
                        // User hasn't modified the file — safe to update
                        FileClassification::AutoUpdate
                    }
                    _ => {
                        // User has modified the file — conflict
                        FileClassification::Conflict
                    }
                }
            }
        }
    }
}

/// Update report
#[derive(Debug, Clone, Default)]
pub struct UpdateReport {
    pub files_updated: Vec<String>,
    pub files_skipped: Vec<String>,
    pub files_conflicted: Vec<String>,
    pub files_created: Vec<String>,
}

impl UpdateReport {
    pub fn summary(&self) -> String {
        format!(
            "📊 更新报告 / Update Report:\n\
             ✅ 已更新 / Updated: {} 个文件 / files\n\
             ⏭️  已跳过 / Skipped: {} 个文件 / files\n\
             ⚠️  冲突 / Conflicts: {} 个文件 / files\n\
             🆕 新增 / Created: {} 个文件 / files",
            self.files_updated.len(),
            self.files_skipped.len(),
            self.files_conflicted.len(),
            self.files_created.len(),
        )
    }
}

/// Project update engine
pub struct UpdateEngine {
    project_dir: PathBuf,
    dry_run: bool,
    force: bool,
    template_dir: Option<PathBuf>,
}

impl UpdateEngine {
    pub fn new(
        project_dir: PathBuf,
        dry_run: bool,
        force: bool,
        template_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            project_dir,
            dry_run,
            force,
            template_dir,
        }
    }

    /// Execute the update workflow
    pub fn update(&self, _interactive: bool) -> Result<UpdateReport> {
        // 1. Read metadata
        let metadata = MetadataManager::read(&self.project_dir)?;

        println!(
            "📋 读取元数据 / Reading metadata: v{}, {} 个文件 / files",
            metadata.version,
            metadata.file_checksums.len()
        );

        // 2. Regenerate templates using stored config
        let resolver = TemplateResolver::new(self.template_dir.clone());
        let resolved = resolver.resolve(metadata.config.mode, metadata.config.ci)?;
        let engine = TemplateEngine::new();
        let ctx = TemplateContext::from_config(&metadata.config);

        // 3. Render all templates to get new content
        let mut new_files: HashMap<String, String> = HashMap::new();
        for (name, template) in &resolved {
            let rendered = engine.render_template(name, &template.content, &ctx)?;
            if !rendered.trim().is_empty() {
                new_files.insert(template.path.clone(), rendered);
            }
        }

        // 4. Classify each file
        let mut report = UpdateReport::default();

        for (file_path, new_content) in &new_files {
            let full_path = self.project_dir.join(file_path);
            let current_content = if full_path.exists() {
                Some(std::fs::read(&full_path)?)
            } else {
                None
            };

            let stored_checksum = metadata.file_checksums.get(file_path).map(|s| s.as_str());

            let classification = if self.force && current_content.is_some() {
                // Force mode: auto-update everything that differs
                if current_content.as_deref() == Some(new_content.as_bytes()) {
                    FileClassification::Skip
                } else {
                    FileClassification::AutoUpdate
                }
            } else {
                classify_file(
                    current_content.as_deref(),
                    new_content.as_bytes(),
                    stored_checksum,
                )
            };

            match classification {
                FileClassification::Skip => {
                    report.files_skipped.push(file_path.clone());
                }
                FileClassification::AutoUpdate => {
                    if !self.dry_run {
                        if let Some(parent) = full_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&full_path, new_content)?;
                    }
                    report.files_updated.push(file_path.clone());
                }
                FileClassification::Conflict => {
                    report.files_conflicted.push(file_path.clone());
                }
                FileClassification::New => {
                    if !self.dry_run {
                        if let Some(parent) = full_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&full_path, new_content)?;
                    }
                    report.files_created.push(file_path.clone());
                }
            }
        }

        // 5. Update metadata (unless dry-run)
        if !self.dry_run && (!report.files_updated.is_empty() || !report.files_created.is_empty()) {
            let all_files: Vec<String> = new_files.keys().cloned().collect();
            let checksums = ChecksumCalculator::calculate_all(&self.project_dir, &all_files)?;
            MetadataManager::update(&self.project_dir, checksums)?;
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_skip() {
        let content = b"hello world";
        let checksum = ChecksumCalculator::calculate(content);
        let result = classify_file(Some(content), content, Some(&checksum));
        assert_eq!(result, FileClassification::Skip);
    }

    #[test]
    fn test_classify_auto_update() {
        let old_content = b"old content";
        let new_content = b"new content";
        let stored_checksum = ChecksumCalculator::calculate(old_content);
        let result = classify_file(Some(old_content), new_content, Some(&stored_checksum));
        assert_eq!(result, FileClassification::AutoUpdate);
    }

    #[test]
    fn test_classify_conflict() {
        let user_modified = b"user modified content";
        let new_content = b"new content";
        let original_checksum = ChecksumCalculator::calculate(b"original content");
        let result = classify_file(Some(user_modified), new_content, Some(&original_checksum));
        assert_eq!(result, FileClassification::Conflict);
    }

    #[test]
    fn test_classify_new_file() {
        let new_content = b"new file content";
        let result = classify_file(None, new_content, None);
        assert_eq!(result, FileClassification::New);
    }

    #[test]
    fn test_update_report_summary() {
        let report = UpdateReport {
            files_updated: vec!["a.rs".into()],
            files_skipped: vec!["b.rs".into(), "c.rs".into()],
            files_conflicted: vec![],
            files_created: vec!["d.rs".into()],
        };
        let summary = report.summary();
        assert!(summary.contains("1"));
        assert!(summary.contains("2"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 6: Update classification correctness
        /// (a) same content → Skip
        /// (b) different content, checksum matches stored → AutoUpdate
        /// (c) different content, checksum differs from stored → Conflict
        /// Feature: v030-template-and-update, Property 6: Update classification correctness
        /// **Validates: Requirements 4.5, 4.6, 6.2, 6.3**
        #[test]
        fn prop_update_classification(
            original in proptest::collection::vec(any::<u8>(), 1..256),
            new_content in proptest::collection::vec(any::<u8>(), 1..256),
            user_modified in prop::bool::ANY,
        ) {
            let stored_checksum = ChecksumCalculator::calculate(&original);

            if original == new_content {
                // (a) Same content → Skip
                let result = classify_file(Some(&original), &new_content, Some(&stored_checksum));
                prop_assert_eq!(result, FileClassification::Skip);
            } else if !user_modified {
                // (b) User hasn't modified: current = original, differs from new → AutoUpdate
                let result = classify_file(Some(&original), &new_content, Some(&stored_checksum));
                prop_assert_eq!(result, FileClassification::AutoUpdate);
            } else {
                // (c) User has modified: current differs from original and from new → Conflict
                let mut modified = original.clone();
                modified.push(0xFF); // ensure it differs from original
                prop_assume!(modified != new_content); // ensure it also differs from new
                let result = classify_file(Some(&modified), &new_content, Some(&stored_checksum));
                prop_assert_eq!(result, FileClassification::Conflict);
            }
        }

        /// Property 6 supplement: New file classification
        /// When current content is None, classification should be New
        /// Feature: v030-template-and-update, Property 6: Update classification correctness
        /// **Validates: Requirements 4.5, 4.6, 6.2, 6.3**
        #[test]
        fn prop_new_file_classification(
            new_content in proptest::collection::vec(any::<u8>(), 1..256),
        ) {
            let result = classify_file(None, &new_content, None);
            prop_assert_eq!(result, FileClassification::New);
        }
    }
}

#[cfg(test)]
mod integration_proptests {
    use super::*;
    use crate::config::ProjectConfig;
    use crate::generator::project::generate_project;
    use proptest::prelude::*;
    use tempfile::TempDir;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]

        /// Property 7: Dry-run does not modify files
        /// For any project, executing update with dry_run=true should not modify any files.
        /// Feature: v030-template-and-update, Property 7: Dry-run does not modify files
        /// **Validates: Requirements 4.8**
        #[test]
        fn prop_dry_run_no_modifications(
            modify_file in prop::bool::ANY,
        ) {
            let temp = TempDir::new().unwrap();
            let project_dir = temp.path().join("dry-run-app");
            let config = ProjectConfig {
                project_name: "dry-run-app".to_string(),
                ..Default::default()
            };

            generate_project(&project_dir, &config, false, false).unwrap();

            // Optionally modify a file to create a diff
            if modify_file {
                let main_rs = project_dir.join("src/main.rs");
                let content = std::fs::read_to_string(&main_rs).unwrap();
                std::fs::write(&main_rs, format!("{}\n// user modification", content)).unwrap();
            }

            // Snapshot all file contents before dry-run
            let snapshot: HashMap<String, Vec<u8>> = walkdir(&project_dir);

            // Run dry-run update
            let engine = UpdateEngine::new(project_dir.clone(), true, false, None);
            let _report = engine.update(false).unwrap();

            // Verify no files changed
            let after: HashMap<String, Vec<u8>> = walkdir(&project_dir);
            prop_assert_eq!(snapshot.len(), after.len(), "File count changed during dry-run");
            for (path, content) in &snapshot {
                prop_assert_eq!(
                    content,
                    &after[path],
                    "File modified during dry-run: {}",
                    path
                );
            }
        }

        /// Property 8: Force overwrites all differing files
        /// With force=true, all files that differ should be updated, zero conflicts.
        /// Feature: v030-template-and-update, Property 8: Force overwrites all differing files
        /// **Validates: Requirements 4.9**
        #[test]
        fn prop_force_overwrites_all(
            _seed in 0u32..10,
        ) {
            let temp = TempDir::new().unwrap();
            let project_dir = temp.path().join("force-app");
            let config = ProjectConfig {
                project_name: "force-app".to_string(),
                ..Default::default()
            };

            generate_project(&project_dir, &config, false, false).unwrap();

            // Modify a file to create a user modification
            let main_rs = project_dir.join("src/main.rs");
            let content = std::fs::read_to_string(&main_rs).unwrap();
            std::fs::write(&main_rs, format!("{}\n// user modification", content)).unwrap();

            // Run force update
            let engine = UpdateEngine::new(project_dir.clone(), false, true, None);
            let report = engine.update(false).unwrap();

            // Force mode should have zero conflicts
            prop_assert!(
                report.files_conflicted.is_empty(),
                "Force update should have zero conflicts, got: {:?}",
                report.files_conflicted
            );
        }
    }

    /// Walk a directory and collect all file contents
    fn walkdir(dir: &std::path::Path) -> HashMap<String, Vec<u8>> {
        let mut files = HashMap::new();
        walk_recursive(dir, dir, &mut files);
        files
    }

    fn walk_recursive(
        base: &std::path::Path,
        current: &std::path::Path,
        files: &mut HashMap<String, Vec<u8>>,
    ) {
        if let Ok(entries) = std::fs::read_dir(current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip .git directory
                    if path.file_name().map(|n| n == ".git").unwrap_or(false) {
                        continue;
                    }
                    walk_recursive(base, &path, files);
                } else {
                    let rel = path
                        .strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    if let Ok(content) = std::fs::read(&path) {
                        files.insert(rel, content);
                    }
                }
            }
        }
    }
}
