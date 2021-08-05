use crate::{
    Library, LibraryCompilationContext, LibraryDependencies, LibraryGitLocation, LibraryLocation,
    LibraryOptions, LibraryTarget,
};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct OpenSSLLibrary {
    location: LibraryLocation,
    options: LibraryOptions,
}

impl Default for OpenSSLLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenSSLLibrary {
    pub fn new() -> Self {
        Self {
            location: LibraryLocation::Git(
                LibraryGitLocation::new("https://github.com/openssl/openssl.git")
                    .tag("OpenSSL_1_1_1k"),
            ),
            options: Default::default(),
        }
    }

    pub fn compiler(&self, options: &LibraryCompilationContext) -> &str {
        match options.target() {
            LibraryTarget::X8664appleDarwin => "darwin64-x86_64-cc",
            LibraryTarget::AArch64appleDarwin => "darwin64-arm64-cc",
            LibraryTarget::X8664pcWindowsMsvc => "VC-WIN64A",
            LibraryTarget::X8664UnknownlinuxGNU => "linux-x86_64-clang",
        }
    }
}

impl Library for OpenSSLLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        "openssl"
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
        let out_dir = self.native_library_prefix(options);
        if !out_dir.exists() {
            std::fs::create_dir_all(&out_dir)
                .unwrap_or_else(|_| panic!("Could not create {:?}", &out_dir));
        }

        let makefile_dir = options.build_root().join(self.name());

        let mut command = Command::new("perl");
        command
            .current_dir(&makefile_dir)
            .arg(self.source_directory(options).join("Configure"))
            .arg(format!("--{}", options.profile()))
            .arg(format!(
                "--prefix={}",
                self.native_library_prefix(options).display()
            ))
            .arg(format!(
                "--openssldir={}",
                self.native_library_prefix(options).display()
            ))
            .arg(self.compiler(options))
            .arg("OPT_LEVEL=3");

        if self.is_static() {
            command.arg("no-shared");
        }

        let configure = command.status().unwrap();

        if !configure.success() {
            panic!("Could not configure {}", self.name());
        }

        let make = Command::new("make")
            .current_dir(&makefile_dir)
            .arg("install_sw")
            .status()
            .unwrap();

        if !make.success() {
            panic!("Could not compile {}", self.name());
        }
        Ok(())
    }

    fn compiled_library_directories(&self, _options: &LibraryCompilationContext) -> Vec<PathBuf> {
        unimplemented!()
    }

    fn ensure_requirements(&self, _options: &LibraryCompilationContext) {
        which::which("make").expect("Could not find `make`");
    }

    fn native_library_prefix(&self, options: &LibraryCompilationContext) -> PathBuf {
        options.build_root().join(self.name()).join("build")
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
