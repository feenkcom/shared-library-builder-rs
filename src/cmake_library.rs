use crate::{
    CompiledLibraryName, Library, LibraryCompilationContext, LibraryDependencies, LibraryLocation,
    LibraryOptions, LibraryTarget,
};
use file_matcher::{FileNamed, FilesNamed};
use rustc_version::version_meta;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CMakeLibrary {
    name: String,
    compiled_name: CompiledLibraryName,
    location: LibraryLocation,
    defines: CMakeLibraryDefines,
    dependencies: LibraryDependencies,
    options: LibraryOptions,
    files_to_delete: Vec<FileNamed>,
}

impl CMakeLibrary {
    pub fn new(name: &str, location: LibraryLocation) -> Self {
        Self {
            name: name.to_owned(),
            compiled_name: CompiledLibraryName::Default,
            location,
            defines: Default::default(),
            dependencies: LibraryDependencies::new(),
            options: Default::default(),
            files_to_delete: vec![],
        }
    }

    fn with_defines(self, defines: CMakeLibraryDefines) -> Self {
        Self {
            name: self.name,
            compiled_name: self.compiled_name,
            location: self.location,
            defines,
            dependencies: self.dependencies,
            options: self.options,
            files_to_delete: self.files_to_delete,
        }
    }

    pub fn defines(&self) -> &CMakeLibraryDefines {
        &self.defines
    }

    pub fn define_common(self, define: impl Into<String>, value: impl Into<String>) -> Self {
        let defines = self.defines().clone().define_common(define, value);
        self.with_defines(defines)
    }

    pub fn define_shared(self, define: impl Into<String>, value: impl Into<String>) -> Self {
        let defines = self.defines().clone().define_shared(define, value);
        self.with_defines(defines)
    }

    pub fn define_static(self, define: impl Into<String>, value: impl Into<String>) -> Self {
        let defines = self.defines().clone().define_static(define, value);
        self.with_defines(defines)
    }

    pub fn depends(self, library: Box<dyn Library>) -> Self {
        Self {
            name: self.name,
            compiled_name: self.compiled_name,
            location: self.location,
            defines: self.defines,
            dependencies: self.dependencies.push(library),
            options: self.options,
            files_to_delete: self.files_to_delete,
        }
    }

    pub fn compiled_name(self, compiled_name: CompiledLibraryName) -> Self {
        Self {
            name: self.name,
            compiled_name,
            location: self.location,
            defines: self.defines,
            dependencies: self.dependencies,
            options: self.options,
            files_to_delete: self.files_to_delete,
        }
    }

    pub fn delete(self, entry_to_delete: impl Into<FileNamed>) -> Self {
        let mut entries = self.files_to_delete;
        entries.push(entry_to_delete.into());

        Self {
            name: self.name,
            compiled_name: self.compiled_name,
            location: self.location,
            defines: self.defines,
            dependencies: self.dependencies,
            options: self.options,
            files_to_delete: entries,
        }
    }
}

impl Library for CMakeLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn compiled_library_name(&self) -> &CompiledLibraryName {
        &self.compiled_name
    }

    fn ensure_sources(&self, options: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        self.location()
            .ensure_sources(&self.source_directory(options), options)?;
        Ok(())
    }

    fn dependencies(&self) -> Option<&LibraryDependencies> {
        Some(&self.dependencies)
    }

    fn options(&self) -> &LibraryOptions {
        &self.options
    }

    fn options_mut(&mut self) -> &mut LibraryOptions {
        &mut self.options
    }
    fn force_compile(&self, options: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        let mut config = cmake::Config::new(self.source_directory(options));

        let out_dir = self.native_library_prefix(options);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir)
                .unwrap_or_else(|_| panic!("Could not create {:?}", &out_dir));
        }

        config
            .static_crt(true)
            .target(&options.target().to_string())
            .host(&version_meta().unwrap().host)
            .out_dir(&out_dir)
            .profile(&options.profile());

        println!(
            "Building CMake library for target = {:?} and host = {:?}",
            &options.target().to_string(),
            &version_meta().unwrap().host
        );

        let mut cmake_prefix_paths = self.all_native_library_prefixes(options);
        if let Ok(ref path) = std::env::var("CMAKE_PREFIX_PATH") {
            cmake_prefix_paths.push(Path::new(path).to_path_buf());
        }

        let cmake_prefix_path = cmake_prefix_paths
            .into_iter()
            .map(|each| each.into_os_string().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(";");

        config.define("CMAKE_PREFIX_PATH", &cmake_prefix_path);

        match options.target() {
            LibraryTarget::X8664appleDarwin => {
                config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
            }
            LibraryTarget::AArch64appleDarwin => {
                config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
            }
            _ => {}
        }

        let ld_library_paths = self
            .all_native_library_prefixes(options)
            .into_iter()
            .map(|each| each.join("lib"))
            .collect::<Vec<PathBuf>>();

        for library_path in &ld_library_paths {
            config.cflag(format!("-L{}", library_path.display()));
        }

        let mut pkg_config_paths = self.all_pkg_config_directories(options);
        if let Ok(ref path) = std::env::var("PKG_CONFIG_PATH") {
            std::env::split_paths(path).for_each(|path| pkg_config_paths.push(path));
        }
        let pkg_config_path = std::env::join_paths(&pkg_config_paths)?;

        config.env("PKG_CONFIG_PATH", &pkg_config_path);

        let mut defines = self.defines.common_defines().clone();
        if self.is_static() {
            defines.extend(self.defines.static_defines().clone())
        } else {
            defines.extend(self.defines.shared_defines().clone())
        }

        for define in defines {
            config.define(&define.0, &define.1);
        }

        config.build();

        for entry_to_delete in &self.files_to_delete {
            let lib = entry_to_delete.within(out_dir.join("lib"));
            std::fs::remove_file(lib.as_path_buf().unwrap()).unwrap();
        }

        if options.is_windows() {
            let libraries = FilesNamed::wildmatch("lib*.lib")
                .within(out_dir.join("lib"))
                .find()?;

            for library in libraries {
                if let Some(file_name) = library.file_name() {
                    if let Some(file_name) = file_name.to_str() {
                        if let Some(new_name) = file_name.strip_prefix("lib") {
                            let copy_as = library.with_file_name(new_name);
                            std::fs::copy(&library, &copy_as)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn compiled_library_directories(&self, options: &LibraryCompilationContext) -> Vec<PathBuf> {
        let lib_dir = self.native_library_prefix(options).join("lib");
        let bin_dir = self.native_library_prefix(options).join("bin");
        vec![lib_dir, bin_dir]
    }

    fn ensure_requirements(&self, _options: &LibraryCompilationContext) {
        which::which("pkg-config")
            .expect("CMake projects require pkg-config, make sure it is installed");
    }

    fn native_library_prefix(&self, context: &LibraryCompilationContext) -> PathBuf {
        context.build_root().join(self.name())
    }

    fn native_library_include_headers(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut dirs = vec![];

        let directory = self.native_library_prefix(context).join("include");

        if directory.exists() {
            dirs.push(directory);
        }

        dirs
    }

    fn native_library_linker_libraries(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        let mut dirs = vec![];

        let directory = self.native_library_prefix(context).join("lib");

        if directory.exists() {
            dirs.push(directory);
        }

        dirs
    }

    fn pkg_config_directory(&self, context: &LibraryCompilationContext) -> Option<PathBuf> {
        let directory = self
            .native_library_prefix(context)
            .join("share")
            .join("pkgconfig");

        if directory.exists() {
            return Some(directory);
        }

        let directory = self
            .native_library_prefix(context)
            .join("lib")
            .join("pkgconfig");

        if directory.exists() {
            return Some(directory);
        }

        None
    }

    fn clone_library(&self) -> Box<dyn Library> {
        Box::new(Clone::clone(self))
    }
}

impl From<CMakeLibrary> for Box<dyn Library> {
    fn from(library: CMakeLibrary) -> Self {
        Box::new(library)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CMakeLibraryDefines {
    common: Vec<(String, String)>,
    when_shared: Vec<(String, String)>,
    when_static: Vec<(String, String)>,
}

impl CMakeLibraryDefines {
    pub fn common_defines(&self) -> &Vec<(String, String)> {
        &self.common
    }

    pub fn shared_defines(&self) -> &Vec<(String, String)> {
        &self.when_shared
    }

    pub fn static_defines(&self) -> &Vec<(String, String)> {
        &self.when_static
    }

    pub fn define_common(self, define: impl Into<String>, value: impl Into<String>) -> Self {
        let mut common = self.common;
        common.push((define.into(), value.into()));

        Self {
            common,
            when_shared: self.when_shared,
            when_static: self.when_static,
        }
    }
    pub fn define_shared(self, define: impl Into<String>, value: impl Into<String>) -> Self {
        let mut when_shared = self.when_shared;
        when_shared.push((define.into(), value.into()));

        Self {
            common: self.common,
            when_shared,
            when_static: self.when_static,
        }
    }

    pub fn define_static(self, define: impl Into<String>, value: impl Into<String>) -> Self {
        let mut when_static = self.when_static;
        when_static.push((define.into(), value.into()));

        Self {
            common: self.common,
            when_shared: self.when_shared,
            when_static,
        }
    }
}
