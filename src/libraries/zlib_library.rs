use crate::{
    CMakeLibrary, CompiledLibraryName, Library, LibraryCompilationContext, LibraryDependencies,
    LibraryGitLocation, LibraryLocation, LibraryOptions,
};
use file_matcher::FileNamed;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ZLibLibrary(CMakeLibrary);

impl ZLibLibrary {
    pub fn default() -> Self {
        Self::version("v1.2.11")
    }

    pub fn version(version: impl Into<String>) -> Self {
        Self(
            CMakeLibrary::new(
                "zlib",
                LibraryLocation::Git(
                    LibraryGitLocation::new("https://github.com/madler/zlib.git").tag(version),
                ),
            )
            .compiled_name(CompiledLibraryName::Matching("zlib".to_string()))
            .define_static("BUILD_SHARED_LIBS", "OFF")
            .define_shared("BUILD_SHARED_LIBS", "ON")
            .delete(FileNamed::any_named(vec![
                FileNamed::wildmatch("*zlib.*"),  // windows
                FileNamed::wildmatch("*.dylib"),  // mac
                FileNamed::wildmatch("libz.so*"), // linux
            ])),
        )
    }
}

impl Library for ZLibLibrary {
    fn location(&self) -> &LibraryLocation {
        self.0.location()
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn dependencies(&self) -> Option<&LibraryDependencies> {
        self.0.dependencies()
    }

    fn options(&self) -> &LibraryOptions {
        self.0.options()
    }

    fn options_mut(&mut self) -> &mut LibraryOptions {
        self.0.options_mut()
    }

    fn force_compile(&self, context: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        self.0.force_compile(context)
    }

    fn compiled_library_directories(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        self.0.compiled_library_directories(context)
    }

    fn ensure_requirements(&self, context: &LibraryCompilationContext) {
        self.0.ensure_requirements(context)
    }

    fn native_library_prefix(&self, context: &LibraryCompilationContext) -> PathBuf {
        self.0.native_library_prefix(context)
    }

    fn native_library_include_headers(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        self.0.native_library_include_headers(context)
    }

    fn native_library_linker_libraries(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        self.0.native_library_linker_libraries(context)
    }

    fn pkg_config_directory(&self, context: &LibraryCompilationContext) -> Option<PathBuf> {
        self.0.pkg_config_directory(context)
    }

    fn clone_library(&self) -> Box<dyn Library> {
        Box::new(Clone::clone(self))
    }
}

impl From<ZLibLibrary> for Box<dyn Library> {
    fn from(library: ZLibLibrary) -> Self {
        Box::new(library)
    }
}
