use crate::{
    BZip2Library, CMakeLibrary, CompiledLibraryName, Library, LibraryCompilationContext,
    LibraryDependencies, LibraryGitLocation, LibraryLocation, LibraryOptions, PngLibrary,
    ZLibLibrary,
};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FreetypeLibrary(CMakeLibrary);

impl FreetypeLibrary {
    pub fn default() -> Self {
        Self::version("VER-2-10-4")
    }

    pub fn version(version: impl Into<String>) -> Self {
        Self(
            CMakeLibrary::new(
                "freetype",
                LibraryLocation::Git(
                    LibraryGitLocation::github("freetype", "freetype").tag(version),
                ),
            )
            .depends(PngLibrary::default().into())
            .depends(ZLibLibrary::default().into())
            .depends(BZip2Library::default().into())
            .define_common("FT_REQUIRE_ZLIB", "TRUE")
            .define_common("FT_REQUIRE_PNG", "TRUE")
            .define_common("FT_REQUIRE_BZIP2", "TRUE")
            .define_shared("BUILD_SHARED_LIBS", "ON")
            .define_static("BUILD_SHARED_LIBS", "OFF")
            .compiled_name(CompiledLibraryName::Matching("freetype".to_string())),
        )
    }
}

impl Library for FreetypeLibrary {
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
        let mut headers = self.0.native_library_include_headers(context);
        headers.push(
            self.native_library_prefix(&context)
                .join("include")
                .join("freetype2"),
        );
        headers
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

impl From<FreetypeLibrary> for Box<dyn Library> {
    fn from(library: FreetypeLibrary) -> Self {
        Box::new(library)
    }
}
