use crate::LibraryCompilationContext;
use std::error::Error;
use std::path::{Path, PathBuf};
use user_error::UserFacingError;

#[derive(Debug, Clone)]
pub struct PathLocation {
    path: PathBuf,
}

impl PathLocation {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub(crate) fn sources_directory(
        &self,
        _default_source_directory: &Path,
        _context: &LibraryCompilationContext,
    ) -> PathBuf {
        self.path.clone()
    }

    pub(crate) fn ensure_sources(
        &self,
        _default_source_directory: &Path,
        _context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        if !self.path.exists() {
            return Err(Box::new(
                UserFacingError::new("Failed to build project")
                    .reason(format!(
                        "{} sources directory does not exist",
                        self.path.display()
                    ))
                    .help("Make sure the configuration is correct and the sources exist"),
            ));
        }
        Ok(())
    }
}
