// Custom template loader
//
// This module loads custom templates from a user-specified directory on the filesystem.

use crate::error::{CliError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Load custom templates from a filesystem directory
pub struct CustomTemplateLoader;

impl CustomTemplateLoader {
    /// Recursively scan a directory and load all `.hbs` files
    ///
    /// Returns HashMap<relative_path, file_content>
    /// The relative path uses forward slashes and strips the `.hbs` extension
    /// to match the built-in template key format.
    ///
    /// # Errors
    /// Returns error if the directory does not exist or is not a directory.
    /// Returns empty map if the directory is empty.
    pub fn load(dir: &Path) -> Result<HashMap<String, String>> {
        if !dir.exists() {
            return Err(CliError::Config(format!(
                "❌ 自定义模板目录不存在 / Custom template directory does not exist: '{}'",
                dir.display()
            )));
        }

        if !dir.is_dir() {
            return Err(CliError::Config(format!(
                "❌ 路径不是目录 / Path is not a directory: '{}'",
                dir.display()
            )));
        }

        let mut templates = HashMap::new();
        Self::scan_dir(dir, dir, &mut templates)?;
        Ok(templates)
    }

    /// Recursively scan directory for .hbs files
    fn scan_dir(
        base: &Path,
        current: &Path,
        templates: &mut HashMap<String, String>,
    ) -> Result<()> {
        let entries = std::fs::read_dir(current).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "❌ 无法读取目录 / Cannot read directory: '{}': {}",
                    current.display(),
                    e
                ),
            ))
        })?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::scan_dir(base, &path, templates)?;
            } else if path.extension().is_some_and(|ext| ext == "hbs") {
                let relative = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");

                // Strip .hbs extension for the key to match built-in template keys
                let key = relative.strip_suffix(".hbs").unwrap_or(&relative);

                let content = std::fs::read_to_string(&path)?;
                templates.insert(key.to_string(), content);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_dir() {
        let result = CustomTemplateLoader::load(Path::new("/nonexistent/dir"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_empty_dir() {
        let temp = TempDir::new().unwrap();
        let result = CustomTemplateLoader::load(temp.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_hbs_files() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml.hbs"), "cargo content").unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/main.rs.hbs"), "main content").unwrap();

        let result = CustomTemplateLoader::load(temp.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["Cargo.toml"], "cargo content");
        assert_eq!(result["src/main.rs"], "main content");
    }

    #[test]
    fn test_load_ignores_non_hbs_files() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("readme.md"), "not a template").unwrap();
        std::fs::write(temp.path().join("template.hbs"), "is a template").unwrap();

        let result = CustomTemplateLoader::load(temp.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("template"));
    }

    #[test]
    fn test_load_file_not_dir() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("not_a_dir.txt");
        std::fs::write(&file_path, "content").unwrap();

        let result = CustomTemplateLoader::load(&file_path);
        assert!(result.is_err());
    }
}
