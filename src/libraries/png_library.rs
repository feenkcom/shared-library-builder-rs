use crate::{
    CMakeLibrary, CompiledLibraryName, Library, LibraryCompilationContext, LibraryDependencies,
    LibraryGitLocation, LibraryLocation, LibraryOptions, ZLibLibrary,
};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PngLibrary(CMakeLibrary);

impl PngLibrary {
    pub fn default() -> Self {
        Self::version("v1.6.37")
    }

    pub fn version(version: impl Into<String>) -> Self {
        Self(
            CMakeLibrary::new(
                "png",
                LibraryLocation::Git(LibraryGitLocation::github("glennrp", "libpng").tag(version)),
            )
            .depends(ZLibLibrary::default().into())
            .compiled_name(CompiledLibraryName::Matching("png".to_string()))
            .define_static("PNG_SHARED", "OFF")
            .define_static("PNG_STATIC", "ON")
            .define_shared("PNG_SHARED", "ON")
            .define_shared("PNG_STATIC", "OFF")
            .define_common("PNG_EXECUTABLES", "OFF")
            .define_common("PNG_TESTS", "OFF")
            .define_common("PNG_ARM_NEON", "off"),
        )
    }
}

impl Library for PngLibrary {
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

impl From<PngLibrary> for Box<dyn Library> {
    fn from(library: PngLibrary) -> Self {
        Box::new(library)
    }
}
