use crate::{Library, LibraryCompilationContext};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct LibraryDependencies {
    dependencies: Vec<Box<dyn Library>>,
}

impl LibraryDependencies {
    pub fn new() -> Self {
        Self {
            dependencies: vec![],
        }
    }

    pub fn push(self, dependency: Box<dyn Library>) -> Self {
        let mut dependencies = self.dependencies;

        let mut static_dependency = dependency.clone_library();
        static_dependency.be_static();
        dependencies.push(static_dependency);
        Self { dependencies }
    }

    pub fn dependency_prefixes(&self, options: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            for each in dependency.all_native_library_prefixes(options) {
                paths.push(each);
            }
            paths.push(dependency.native_library_prefix(options));
        }
        paths
    }

    pub fn include_headers(&self, options: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            if let Some(dependencies) = dependency.dependencies() {
                paths.extend(dependencies.include_headers(options));
            }
            paths.extend(dependency.native_library_include_headers(options));
        }
        paths
    }

    pub fn linker_libraries(&self, options: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            for each in dependency.native_library_linker_libraries(options) {
                paths.push(each);
            }
        }
        paths
    }

    pub fn include_headers_flags(&self, options: &LibraryCompilationContext) -> String {
        self.include_headers(options)
            .into_iter()
            .map(|path| format!("-I{}", path.display()))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn linker_libraries_flags(&self, options: &LibraryCompilationContext) -> String {
        self.linker_libraries(options)
            .into_iter()
            .map(|path| format!("-L{}", path.display()))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn ensure_sources(
        &self,
        options: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        for dependency in &self.dependencies {
            if let Some(dependencies) = dependency.dependencies() {
                dependencies.ensure_sources(options)?;
            }
            dependency.ensure_sources(options)?;
        }
        Ok(())
    }

    pub fn ensure_requirements(
        &self,
        options: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        for dependency in &self.dependencies {
            if let Some(dependencies) = dependency.dependencies() {
                dependencies.ensure_requirements(options)?;
            }
            dependency.ensure_requirements(options);
        }
        Ok(())
    }

    pub fn force_compile(&self, options: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        for dependency in &self.dependencies {
            if let Some(dependencies) = dependency.dependencies() {
                dependencies.force_compile(options)?;
            }
            dependency.force_compile(options)?;
        }
        Ok(())
    }

    pub fn pkg_config_directories(&self, options: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut paths = vec![];
        for dependency in &self.dependencies {
            if let Some(ref path) = dependency.pkg_config_directory(options) {
                paths.push(path.clone());
            }
            paths.extend(dependency.all_pkg_config_directories(options));
        }
        paths
    }

    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }
}

impl Clone for LibraryDependencies {
    fn clone(&self) -> Self {
        Self {
            dependencies: self
                .dependencies
                .iter()
                .map(|library| library.clone_library())
                .collect::<Vec<Box<dyn Library>>>(),
        }
    }
}
