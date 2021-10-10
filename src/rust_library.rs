use crate::{
    Library, LibraryCompilationContext, LibraryDependencies, LibraryLocation, LibraryOptions,
};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct RustLibrary {
    name: String,
    location: LibraryLocation,
    features: Vec<String>,
    requires: Vec<String>,
    options: LibraryOptions,
    package: Option<String>,
}

impl RustLibrary {
    pub fn new(name: &str, location: LibraryLocation) -> Self {
        Self {
            name: name.to_owned(),
            location,
            features: vec![],
            requires: vec![],
            options: Default::default(),
            package: None,
        }
    }

    pub fn feature(self, feature: impl Into<String>) -> Self {
        let mut library = self;
        library.features.push(feature.into());
        library
    }

    pub fn requires(self, executable: impl Into<String>) -> Self {
        let mut library = self;
        library.requires.push(executable.into());
        library
    }

    pub fn features(self, features: Vec<&str>) -> Self {
        let mut library = self;
        library.features = features.iter().map(|each| each.to_string()).collect();
        library
    }

    pub fn package(self, package: impl Into<String>) -> Self {
        let mut library = self;
        library.package = Some(package.into());
        library
    }

    fn crate_source_directory(&self, context: &LibraryCompilationContext) -> PathBuf {
        self.source_directory(context)
    }
}

impl Library for RustLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn dependencies(&self) -> Option<&LibraryDependencies> {
        None
    }

    fn options(&self) -> &LibraryOptions {
        &self.options
    }

    fn options_mut(&mut self) -> &mut LibraryOptions {
        &mut self.options
    }

    fn force_compile(&self, options: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        let mut command = Command::new("cargo");
        command.arg("build");

        if let Some(package) = &self.package {
            command.arg("--package").arg(package);
        }

        command
            .arg("--lib")
            .arg("--target")
            .arg(options.target().to_string())
            .arg("--target-dir")
            .arg(options.build_root())
            .arg("--manifest-path")
            .arg(self.crate_source_directory(options).join("Cargo.toml"));

        if options.is_release() {
            command.arg("--release");
        }

        let status = command.status().unwrap();
        if !status.success() {
            panic!("Could not compile {}", self.name);
        }
        Ok(())
    }

    fn compiled_library_directories(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        let path = context
            .build_root()
            .join(context.target().to_string())
            .join(context.profile());
        vec![path]
    }

    fn ensure_requirements(&self, _options: &LibraryCompilationContext) {
        self.requires.iter().for_each(|each| {
            which::which(each).unwrap_or_else(|_| {
                panic!(
                    "{} must exist in the system. Make sure it is in the PATH",
                    each
                )
            });
        })
    }

    fn native_library_prefix(&self, context: &LibraryCompilationContext) -> PathBuf {
        context.build_root().to_path_buf()
    }

    fn native_library_include_headers(&self, _context: &LibraryCompilationContext) -> Vec<PathBuf> {
        vec![]
    }

    fn native_library_linker_libraries(
        &self,
        _context: &LibraryCompilationContext,
    ) -> Vec<PathBuf> {
        vec![]
    }

    fn pkg_config_directory(&self, _context: &LibraryCompilationContext) -> Option<PathBuf> {
        None
    }

    fn clone_library(&self) -> Box<dyn Library> {
        Box::new(Clone::clone(self))
    }
}

impl From<RustLibrary> for Box<dyn Library> {
    fn from(library: RustLibrary) -> Self {
        Box::new(library)
    }
}
