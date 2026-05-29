use crate::{Library, LibraryCompilationContext};
use std::error::Error;
use std::path::{Path, PathBuf};
use user_error::UserFacingError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathLocation {
    path: PathBuf,
    #[serde(default)]
    prebuilt_library_directory: Option<PathBuf>,
}

impl PathLocation {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            prebuilt_library_directory: None,
        }
    }

    pub fn prebuilt_library_directory(self, path: impl Into<PathBuf>) -> Self {
        let mut location = self;
        location.prebuilt_library_directory = Some(path.into());
        location
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

    pub(crate) fn retrieve_prebuilt_library(
        &self,
        library: Box<dyn Library>,
        context: &LibraryCompilationContext,
    ) -> Option<PathBuf> {
        let prebuilt_library = self.find_prebuilt_library(library.as_ref(), context)?;
        let exported_library = library.exported_library_path(context);

        if let Some(exported_directory) = exported_library.parent() {
            std::fs::create_dir_all(exported_directory).ok()?;
        }

        if prebuilt_library != exported_library {
            std::fs::copy(prebuilt_library, &exported_library).ok()?;
        }
        Some(exported_library)
    }

    fn find_prebuilt_library(
        &self,
        library: &dyn Library,
        context: &LibraryCompilationContext,
    ) -> Option<PathBuf> {
        let directory = self.prebuilt_library_directory.as_ref()?;
        let asset_name = library.prebuilt_library_asset_name(context);
        let prebuilt_library = directory.join(asset_name);

        prebuilt_library.is_file().then_some(prebuilt_library)
    }
}
