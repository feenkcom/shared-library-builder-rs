use crate::{Library, LibraryCompilationContext, PathLocation};
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum LibraryLocation {
    #[cfg(feature = "git-location")]
    Git(crate::GitLocation),
    Path(PathLocation),
    #[cfg(feature = "tar-location")]
    Tar(crate::TarUrlLocation),
    #[cfg(feature = "zip-location")]
    Zip(crate::ZipUrlLocation),
}

impl LibraryLocation {
    #[cfg(feature = "git-location")]
    pub fn github(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self::Git(crate::GitLocation::github(owner, repo))
    }

    pub fn sources_directory(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> PathBuf {
        match self {
            #[cfg(feature = "git-location")]
            LibraryLocation::Git(git) => git.sources_directory(default_source_directory, context),
            LibraryLocation::Path(path) => {
                path.sources_directory(default_source_directory, context)
            }
            #[cfg(feature = "tar-location")]
            LibraryLocation::Tar(tar) => tar.sources_directory(default_source_directory, context),
            #[cfg(feature = "zip-location")]
            LibraryLocation::Zip(zip) => zip.sources_directory(default_source_directory, context),
        }
    }

    /// Try to retrieve a prebuilt library for the current target and return a path to the file
    #[allow(unused_variables)]
    pub fn retrieve_prebuilt_library(
        &self,
        library: Box<dyn Library>,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Option<PathBuf> {
        // Static libraries must be compiled from sources
        if library.is_static() {
            return None;
        }

        match self {
            #[cfg(feature = "git-location")]
            LibraryLocation::Git(git_location) => {
                git_location.retrieve_prebuilt_library(library, default_source_directory, context)
            }
            _ => None,
        }
    }

    pub fn ensure_sources(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "git-location")]
            LibraryLocation::Git(git_location) => {
                git_location.ensure_sources(default_source_directory, context)
            }
            LibraryLocation::Path(path_location) => {
                path_location.ensure_sources(default_source_directory, context)
            }
            #[cfg(feature = "tar-location")]
            LibraryLocation::Tar(tar_location) => {
                tar_location.ensure_sources(default_source_directory, context)
            }
            #[cfg(feature = "zip-location")]
            LibraryLocation::Zip(zip_location) => {
                zip_location.ensure_sources(default_source_directory, context)
            }
        }
    }
}
