use crate::{
    Library, LibraryCompilationContext, LibraryDependencies, LibraryLocation, LibraryOptions,
};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RustLibrary {
    name: String,
    location: LibraryLocation,
    features: Vec<String>,
    requires: Vec<String>,
    env_vars: Vec<(OsString, OsString)>,
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
            env_vars: vec![],
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

    pub fn env(self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        let mut library = self;
        library.env_vars.push((key.into(), value.into()));
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

    fn current_dir_contains_crate(&self) -> Result<bool, Box<dyn Error>> {
        let manifest = env::current_dir()?.join("Cargo.toml");
        if !manifest.exists() {
            return Ok(false);
        }

        let manifest_contents = fs::read_to_string(&manifest)?;
        let cargo_toml: toml::Value = toml::from_str(&manifest_contents)?;

        if self.manifest_matches(&cargo_toml) {
            return Ok(true);
        }

        if let Some(workspace) = cargo_toml.get("workspace") {
            if let Some(members) = workspace
                .get("members")
                .and_then(|members| members.as_array())
            {
                let root = manifest
                    .parent()
                    .map(|parent| parent.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."));

                for member in members {
                    if let Some(member) = member.as_str() {
                        if self.member_manifest_matches(&root, member)? {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    fn member_manifest_matches(&self, root: &Path, member: &str) -> Result<bool, Box<dyn Error>> {
        let pattern = root.join(member).join("Cargo.toml");
        let pattern = pattern.to_string_lossy().to_string();

        for manifest in glob(&pattern)? {
            let manifest = manifest?;
            if !manifest.exists() {
                continue;
            }

            let manifest_contents = fs::read_to_string(&manifest)?;
            let cargo_toml: toml::Value = toml::from_str(&manifest_contents)?;

            if self.manifest_matches(&cargo_toml) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn manifest_matches(&self, manifest: &toml::Value) -> bool {
        if let Some(package_name) = &self.package {
            let manifest_package = manifest
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(|name| name.as_str());

            if Some(package_name.as_str()) == manifest_package {
                return true;
            }
        }

        let lib_name = manifest
            .get("lib")
            .and_then(|lib| lib.get("name"))
            .and_then(|name| name.as_str());

        lib_name == Some(self.name())
    }
}

#[typetag::serde]
impl Library for RustLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn source_directory(&self, context: &LibraryCompilationContext) -> PathBuf {
        if self.current_dir_contains_crate().unwrap_or(false) {
            return env::current_dir().unwrap();
        }
        self.location()
            .sources_directory(&PathBuf::from(self.name()), context)
    }

    fn ensure_sources(&self, context: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        if self.current_dir_contains_crate().unwrap_or(false) {
            return Ok(());
        }

        let location = self.location();
        location.ensure_sources(&PathBuf::from(self.name()), context)
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

    fn force_compile(&self, context: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        let mut command = Command::new("cargo");

        command.envs(
            self.env_vars
                .iter()
                .map(|(k, v)| (k.as_os_str(), v.as_os_str())),
        );

        if context.is_android() {
            command.arg("apk").arg("--");
        }

        command.arg("build");

        if let Some(package) = &self.package {
            command.arg("--package").arg(package);
        }

        command.arg("--lib");

        if !context.target().is_current() {
            command.arg("--target").arg(context.target().to_string());
        }
        command
            .arg("--target-dir")
            .arg(context.build_root())
            .arg("--manifest-path")
            .arg(self.crate_source_directory(context).join("Cargo.toml"));

        if !self.features.is_empty() {
            command.arg("--features").arg(self.features.join(" "));
        }

        if context.is_release() {
            command.arg("--release");
        }

        if context.is_windows() {
            command.env("RUSTFLAGS", "-C target-feature=+crt-static");
        }
        if context.is_mac() {
            let version = context.macos_target_version();
            command.env(
                "RUSTFLAGS",
                format!(
                    "-C link-arg=-mmacosx-version-min={} -C link-arg=-Wl,-headerpad,{}",
                    &version,
                    context.macos_headerpad()
                ),
            );
            command.env("MACOSX_DEPLOYMENT_TARGET", &version);
        }

        let status = command.status().unwrap();
        if !status.success() {
            panic!("Could not compile {}", self.name);
        }
        Ok(())
    }

    fn compiled_library_directories(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        let path_without_target = context.build_root().join(context.profile());

        let path_with_target = context
            .build_root()
            .join(context.target().to_string())
            .join(context.profile());

        if context.target().is_current() {
            vec![path_without_target]
        } else {
            vec![path_with_target]
        }
    }

    fn ensure_requirements(&self, _options: &LibraryCompilationContext) {
        self.requires.iter().for_each(|each| {
            which::which(each).unwrap_or_else(|_| {
                let key = "PATH";
                match std::env::var_os(key) {
                    Some(paths) => {
                        println!("PATH:");
                        for path in std::env::split_paths(&paths) {
                            println!("  '{}'", path.display());
                        }
                    }
                    None => println!("{} is not defined in the environment.", key),
                }
                panic!(
                    "{} must exist in the system. Make sure it is in the {}",
                    each, key
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
