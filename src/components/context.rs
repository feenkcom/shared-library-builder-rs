use crate::LibraryTarget;
use std::path::{Path, PathBuf};

pub const DEFAULT_MACOSX_DEPLOYMENT_TARGET_X86_64: &str = "10.10";
pub const DEFAULT_MACOSX_DEPLOYMENT_TARGET_AARCH64: &str = "11.0";
pub const DEFAULT_ANDROID_TARGET_API: &str = "30";

#[derive(Debug, Clone)]
pub struct LibraryCompilationContext {
    sources_root: PathBuf,
    build_root: PathBuf,
    target: LibraryTarget,
    debug: bool,
    macos_target_version: Option<String>,
    android_target_api: Option<String>,
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
            macos_target_version: None,
            android_target_api: None,
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
            macos_target_version: None,
            android_target_api: None,
        }
    }

    pub fn macos_target_version(&self) -> String {
        self.macos_target_version
            .clone()
            .or_else(|| std::env::var("MACOSX_DEPLOYMENT_TARGET").ok())
            .unwrap_or_else(|| {
                (match self.target() {
                    LibraryTarget::AArch64appleDarwin => DEFAULT_MACOSX_DEPLOYMENT_TARGET_AARCH64,
                    _ => DEFAULT_MACOSX_DEPLOYMENT_TARGET_X86_64,
                })
                .to_string()
            })
    }

    pub fn android_target_api(&self) -> String {
        self.android_target_api
            .clone()
            .or_else(|| std::env::var("ANDROID_TARGET_API").ok())
            .unwrap_or_else(|| DEFAULT_ANDROID_TARGET_API.to_string())
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
