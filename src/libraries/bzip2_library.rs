use crate::{
    CMakeLibrary, CompiledLibraryName, Library, LibraryCompilationContext, LibraryDependencies,
    LibraryGitLocation, LibraryLocation, LibraryOptions,
};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BZip2Library(CMakeLibrary);

impl BZip2Library {
    pub fn default() -> Self {
        Self::commit("bf905ea2251191ff9911ae7ec0cfc35d41f9f7f6")
    }

    pub fn commit(commit: impl Into<String>) -> Self {
        Self(
            CMakeLibrary::new(
                "bzip2",
                LibraryLocation::Git(
                    LibraryGitLocation::gitlab("federicomenaquintero", "bzip2").commit(commit),
                ),
            )
            .compiled_name(CompiledLibraryName::Matching("bzip2".to_string()))
            .define_common("ENABLE_LIB_ONLY", "ON")
            .define_static("ENABLE_STATIC_LIB", "ON")
            .define_static("BUILD_SHARED_LIBS", "OFF")
            .define_static("ENABLE_SHARED_LIB", "OFF")
            .define_shared("ENABLE_STATIC_LIB", "OFF")
            .define_shared("ENABLE_SHARED_LIB", "ON")
            .define_shared("BUILD_SHARED_LIBS", "ON"),
        )
    }
}

impl Library for BZip2Library {
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

impl From<BZip2Library> for Box<dyn Library> {
    fn from(library: BZip2Library) -> Self {
        Box::new(library)
    }
}
