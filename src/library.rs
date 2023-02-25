use crate::{
    LibraryCompilationContext, LibraryDependencies, LibraryLocation, LibraryOptions, LibraryTarget,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use user_error::UserFacingError;

#[typetag::serde(tag = "type")]
pub trait Library: Debug + Send + Sync {
    fn location(&self) -> &LibraryLocation;
    fn release_location(&self) -> &LibraryLocation {
        self.location()
    }
    fn name(&self) -> &str;
    fn compiled_library_name(&self) -> CompiledLibraryName {
        CompiledLibraryName::Default
    }

    /// Return a name of the library when exporting as a shared library.
    /// By default it is the same as a general library name, however, some libraries may have a custom naming,
    /// for example zlib -> libz
    fn exported_name(&self) -> &str {
        self.name()
    }

    fn source_directory(&self, context: &LibraryCompilationContext) -> PathBuf {
        self.location()
            .sources_directory(&PathBuf::from(self.name()), context)
    }

    fn ensure_sources(&self, context: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        let location = self.location();
        location.ensure_sources(&PathBuf::from(self.name()), context)
    }

    fn retrieve_prebuilt_library(&self, context: &LibraryCompilationContext) -> Option<PathBuf> {
        let location = self.release_location();
        location.retrieve_prebuilt_library(
            self.clone_library(),
            &PathBuf::from(self.name()),
            context,
        )
    }

    fn dependencies(&self) -> Option<&LibraryDependencies>;
    fn options(&self) -> &LibraryOptions;
    fn options_mut(&mut self) -> &mut LibraryOptions;

    fn is_static(&self) -> bool {
        self.options().is_static()
    }

    fn is_shared(&self) -> bool {
        !self.is_static()
    }

    fn be_static(&mut self) {
        self.options_mut().be_static(true);
    }

    fn be_shared(&mut self) {
        self.options_mut().be_static(false);
    }

    fn is_compiled(&self, context: &LibraryCompilationContext) -> bool {
        self.compiled_library(context).exists()
    }

    fn compile(&self, context: &LibraryCompilationContext) -> Result<PathBuf, Box<dyn Error>> {
        if let Some(prebuilt_library) = self.retrieve_prebuilt_library(context) {
            return Ok(prebuilt_library);
        }

        if let Some(dependencies) = self.dependencies() {
            dependencies.ensure_requirements(context)?;
        }

        self.ensure_requirements(context);

        if let Some(dependencies) = self.dependencies() {
            dependencies.ensure_sources(context)?;
        }
        self.ensure_sources(context)?;
        if let Some(dependencies) = self.dependencies() {
            dependencies.force_compile(context)?;
        }

        println!("About to build {} from\n{:?}", self.name(), self);
        self.force_compile(context)?;

        if self.is_shared() {
            self.export_compiled_library(context)
        } else {
            Ok(self.compiled_library(context))
        }
    }

    fn force_compile(&self, context: &LibraryCompilationContext) -> Result<(), Box<dyn Error>>;

    fn compiled_library_directories(&self, context: &LibraryCompilationContext) -> Vec<PathBuf>;

    fn export_compiled_library(
        &self,
        context: &LibraryCompilationContext,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let compiled_library = self.compiled_library(context);

        let mut exported_path = context
            .build_root()
            .join(context.target().to_string())
            .join(context.profile());

        if !exported_path.exists() {
            std::fs::create_dir_all(&exported_path)?;
        }

        exported_path = exported_path.join(
            self.compiled_library_name()
                .file_name(self.exported_name(), context.target()),
        );

        // prevent from overwriting
        if exported_path != compiled_library {
            std::fs::copy(compiled_library, &exported_path)?;
        }

        Ok(exported_path)
    }

    fn compiled_library(&self, context: &LibraryCompilationContext) -> PathBuf {
        let library_name = self.name();
        let compiled_library_name = self.compiled_library_name();
        for directory in self.compiled_library_directories(context) {
            if let Ok(dir) = directory.read_dir() {
                let libraries = dir
                    .filter(|each| each.is_ok())
                    .map(|each| each.unwrap())
                    .filter(|each| each.path().is_file())
                    .filter(|each| {
                        compiled_library_name.matches(library_name, &each.path(), context.target())
                    })
                    .map(|each| each.path())
                    .collect::<Vec<PathBuf>>();

                if !libraries.is_empty() {
                    return libraries.get(0).unwrap().clone();
                }
            }
        }

        panic!("Could not find a compiled library for {}", self.name())
    }

    fn compiled_library_binary(
        &self,
        _context: &LibraryCompilationContext,
    ) -> Result<PathBuf, Box<dyn Error>> {
        Err(UserFacingError::new("Could not find compiled library").into())
    }

    fn ensure_requirements(&self, context: &LibraryCompilationContext);

    /// Return the root build directory of the library.
    fn native_library_prefix(&self, context: &LibraryCompilationContext) -> PathBuf;

    /// Returns a collection of include directories exported by the native library.
    /// Dependent libraries will search headers within these directories
    fn native_library_include_headers(&self, context: &LibraryCompilationContext) -> Vec<PathBuf>;

    /// Returns a collection of directories that contain the compiled libraries.
    /// Dependent libraries will search libraries to link within these directories.
    fn native_library_linker_libraries(&self, context: &LibraryCompilationContext) -> Vec<PathBuf>;

    /// If a native library creates a pkg-config .pc file, return a directory that contains it
    fn pkg_config_directory(&self, context: &LibraryCompilationContext) -> Option<PathBuf>;

    /// Return true if this native library has dependencies
    fn has_dependencies(&self, _context: &LibraryCompilationContext) -> bool {
        self.dependencies()
            .map_or(false, |dependencies| !dependencies.is_empty())
    }

    fn linker_libraries(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut dirs = self.native_library_linker_libraries(context);
        if let Some(dependencies) = self.dependencies() {
            dirs.extend(dependencies.linker_libraries(context));
        }
        dirs
    }

    /// Return all pkg-config directories of all dependencies
    fn all_pkg_config_directories(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut dirs = vec![];
        if let Some(dependencies) = self.dependencies() {
            dirs.extend(dependencies.pkg_config_directories(context));
        }
        dirs
    }

    /// Return all library prefixes (root of the build) of all dependencies
    fn all_native_library_prefixes(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut dirs = vec![];
        if let Some(dependencies) = self.dependencies() {
            dirs.extend(dependencies.dependency_prefixes(context));
        }
        dirs
    }

    fn msvc_include_directories(&self) -> Vec<PathBuf> {
        let msvc = PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\MSVC\\14.29.30037");
        let sdk = PathBuf::from("C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.19041.0");

        vec![
            msvc.join("include"),
            sdk.join("ucrt"),
            sdk.join("shared"),
            sdk.join("um"),
        ]
    }

    fn msvc_lib_directories(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from("C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.19041.0\\um\\x64"),
            PathBuf::from("C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.19041.0\\ucrt\\x64"),
            PathBuf::from("C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\MSVC\\14.29.30037\\lib\\x64")
        ]
    }

    fn clone_library(&self) -> Box<dyn Library>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompiledLibraryName {
    /// same as Library::name
    Default,
    /// find a platform specific library with a name that includes String
    Matching(String),
}

impl CompiledLibraryName {
    pub fn platform_library_ending(&self, target: &LibraryTarget) -> String {
        if target.is_linux() {
            return "so".to_string();
        }
        if target.is_mac() {
            return "dylib".to_string();
        }

        if target.is_android() {
            return "so".to_string();
        }

        if target.is_windows() {
            return "dll".to_string();
        }

        panic!("Unsupported target: {}", target)
    }

    fn platform_library_name(&self, name: &str, target: &LibraryTarget) -> String {
        if target.is_unix() {
            return format!("lib{}.{}", name, self.platform_library_ending(target));
        }
        if target.is_windows() {
            return format!("{}.{}", name, self.platform_library_ending(target));
        }

        panic!("Unsupported target: {}", target)
    }

    pub fn file_name(&self, library_name: &str, target: &LibraryTarget) -> String {
        self.platform_library_name(library_name, target)
    }

    pub fn matches(&self, library_name: &str, path: &Path, target: &LibraryTarget) -> bool {
        match path.file_name() {
            None => false,
            Some(actual_name) => match actual_name.to_str() {
                None => false,
                Some(actual_name) => match self {
                    CompiledLibraryName::Default => {
                        let expected_name = self.platform_library_name(library_name, target);
                        actual_name.eq_ignore_ascii_case(&expected_name)
                    }
                    CompiledLibraryName::Matching(substring) => {
                        actual_name.contains(&format!(".{}", self.platform_library_ending(target)))
                            && actual_name.contains(substring)
                    }
                },
            },
        }
    }
}
