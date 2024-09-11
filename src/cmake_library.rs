use std::collections::HashMap;
use crate::{
    CompiledLibraryName, Library, LibraryCompilationContext, LibraryDependencies, LibraryLocation,
    LibraryOptions, LibraryTarget,
};
use cmake::Config;
use file_matcher::{FileNamed, FilesNamed};
use rustc_version::version_meta;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::format;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CMakeLibrary {
    name: String,
    exported_name: Option<String>,
    compiled_name: CompiledLibraryName,
    source_location: LibraryLocation,
    release_location: Option<LibraryLocation>,
    defines: CMakeLibraryDefines,
    dependencies: LibraryDependencies,
    options: LibraryOptions,
    env_vars: HashMap<OsString, OsString>,
    files_to_delete_static: Vec<FileNamed>,
    header_directories: Vec<PathBuf>,
}

impl CMakeLibrary {
    pub fn new(name: &str, location: LibraryLocation) -> Self {
        Self {
            name: name.to_owned(),
            exported_name: None,
            compiled_name: CompiledLibraryName::Default,
            source_location: location,
            release_location: None,
            defines: Default::default(),
            dependencies: LibraryDependencies::new(),
            options: Default::default(),
            env_vars: Default::default(),
            files_to_delete_static: vec![],
            header_directories: vec![Path::new("include").to_path_buf()],
        }
    }

    fn with_defines(mut self, defines: CMakeLibraryDefines) -> Self {
        self.defines = defines;
        self
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

    pub fn depends(mut self, library: Box<dyn Library>) -> Self {
        self.dependencies = self.dependencies.push(library);
        self
    }

    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    pub fn compiled_name(mut self, compiled_name: CompiledLibraryName) -> Self {
        self.compiled_name = compiled_name;
        self
    }

    pub fn with_exported_name(mut self, exported_name: impl Into<String>) -> Self {
        self.exported_name = Some(exported_name.into());
        self
    }

    /// Set a file to delete when building a static library
    pub fn delete(mut self, entry_to_delete: impl Into<FileNamed>) -> Self {
        self.files_to_delete_static.push(entry_to_delete.into());
        self
    }

    pub fn with_release_location(mut self, release_location: Option<LibraryLocation>) -> Self {
        self.release_location = release_location;
        self
    }

    pub fn with_headers(mut self, header_directory: impl Into<PathBuf>) -> Self {
        self.header_directories.push(header_directory.into());
        self
    }
}

#[typetag::serde]
impl Library for CMakeLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.source_location
    }

    fn release_location(&self) -> &LibraryLocation {
        self.release_location
            .as_ref()
            .unwrap_or_else(|| &self.source_location)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn compiled_library_name(&self) -> CompiledLibraryName {
        self.compiled_name.clone()
    }

    fn exported_name(&self) -> &str {
        self.exported_name
            .as_ref()
            .map(|name| name.as_str())
            .unwrap_or_else(|| self.name())
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
    fn force_compile(&self, context: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        let mut config = cmake::Config::new(self.source_directory(context));

        let out_dir = self.native_library_prefix(context);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir)
                .unwrap_or_else(|_| panic!("Could not create {:?}", &out_dir));
        }

        config
            .static_crt(true)
            .target(&context.target().to_string())
            .host(&version_meta().unwrap().host)
            .out_dir(&out_dir)
            .profile(&context.profile());

        println!(
            "Building CMake library for target = {:?} and host = {:?}",
            &context.target().to_string(),
            &version_meta().unwrap().host
        );

        let mut cmake_prefix_paths = self.all_native_library_prefixes(context);
        if let Ok(ref path) = std::env::var("CMAKE_PREFIX_PATH") {
            cmake_prefix_paths.push(Path::new(path).to_path_buf());
        }

        let cmake_prefix_path = cmake_prefix_paths
            .into_iter()
            .map(|each| each.into_os_string().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(";");

        config.define("CMAKE_PREFIX_PATH", &cmake_prefix_path);

        match context.target() {
            LibraryTarget::X8664appleDarwin => {
                config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
            }
            LibraryTarget::AArch64appleDarwin => {
                config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
            }
            _ => {}
        }

        if context.is_mac() {
            config.env("MACOSX_DEPLOYMENT_TARGET", context.macos_target_version());
        }

        if context.is_android() {
            configure_android_path(&mut config, context);
        }

        let ld_library_paths = self
            .all_native_library_prefixes(context)
            .into_iter()
            .map(|each| each.join("lib"))
            .collect::<Vec<PathBuf>>();

        for library_path in &ld_library_paths {
            config.cflag(format!("-L{}", library_path.display()));
        }

        let mut pkg_config_paths = vec![];
        if let Ok(ref path) = std::env::var("PKG_CONFIG_PATH") {
            pkg_config_paths.extend(std::env::split_paths(path));
        }
        pkg_config_paths.extend(self.all_pkg_config_directories(context));
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

        for (k, v) in self.env_vars.iter() {
            config.env(k, v);
        }

        for (k, v) in self.all_native_library_vars(context) {
            println!("{:?}: {:?}", &k, &v);
            config.define(k, v);
        }

        config.build();

        if self.is_static() {
            for entry_to_delete in &self.files_to_delete_static {
                let lib = entry_to_delete.within(out_dir.join("lib"));
                std::fs::remove_file(lib.as_path_buf().unwrap()).unwrap();
            }
        }

        if context.is_windows() {
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

        for header_dir in &self.header_directories {
            let directory = self.native_library_prefix(context).join(header_dir);

            if directory.exists() {
                dirs.push(directory);
            }
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

fn configure_android_path(config: &mut Config, context: &LibraryCompilationContext) {
    let ndk = ndk_build::ndk::Ndk::from_env().unwrap();

    let new_path = format!(
        "{}:{}",
        ndk.toolchain_dir().unwrap().join("bin").display(),
        std::env::var("PATH").expect("PATH must be set")
    );

    config.env("PATH", new_path);

    let ndk_root = std::env::var("ANDROID_NDK")
        .or_else(|_| std::env::var("NDK_HOME"))
        .expect("ANDROID_NDK or NDK_HOME must be defined");

    config.env("ANDROID_NDK_ROOT", &ndk_root);
    config.define("ANDROID_PLATFORM", format!("android-{}",context.android_target_api()));
    config.define("ANDROID_ABI", "arm64-v8a");

    config.define("CMAKE_SYSTEM_VERSION", context.android_target_api());
    config.define("CMAKE_SYSTEM_NAME", "Android");
    config.define("CMAKE_ANDROID_ARCH_ABI", "arm64-v8a");
    config.define("CMAKE_FIND_ROOT_PATH_MODE_LIBRARY", "BOTH");
}
