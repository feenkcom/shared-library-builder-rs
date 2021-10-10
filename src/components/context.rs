use crate::LibraryTarget;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LibraryCompilationContext {
    sources_root: PathBuf,
    build_root: PathBuf,
    target: LibraryTarget,
    debug: bool,
}

impl LibraryCompilationContext {
    pub fn new(
        sources_root: impl AsRef<Path>,
        build_root: impl AsRef<Path>,
        target: LibraryTarget,
        debug: bool,
    ) -> Self {
        let sources_root = to_absolute::canonicalize(sources_root.as_ref()).unwrap_or_else(|_| {
            panic!("Failed to canonicalize {}", sources_root.as_ref().display())
        });

        let build_root = to_absolute::canonicalize(build_root.as_ref())
            .unwrap_or_else(|_| panic!("Failed to canonicalize {}", build_root.as_ref().display()));

        Self {
            sources_root,
            build_root,
            target,
            debug,
        }
    }

    pub fn new_release(root: impl AsRef<Path>) -> Self {
        let root = to_absolute::canonicalize(root.as_ref())
            .unwrap_or_else(|_| panic!("Failed to canonicalize {}", root.as_ref().display()));

        Self {
            sources_root: root.join("src"),
            build_root: root.join("build"),
            target: LibraryTarget::for_current_platform(),
            debug: false,
        }
    }

    pub fn sources_root(&self) -> &Path {
        &self.sources_root
    }

    pub fn build_root(&self) -> &Path {
        &self.build_root
    }

    pub fn target(&self) -> &LibraryTarget {
        &self.target
    }

    pub fn is_unix(&self) -> bool {
        self.target().is_unix()
    }
    pub fn is_mac(&self) -> bool {
        self.target().is_mac()
    }
    pub fn is_windows(&self) -> bool {
        self.target().is_windows()
    }

    pub fn profile(&self) -> &str {
        if self.debug {
            "debug"
        } else {
            "release"
        }
    }

    pub fn is_release(&self) -> bool {
        !self.debug
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}
