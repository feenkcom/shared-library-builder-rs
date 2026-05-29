use serde::{Deserialize, Serialize};
use shared_library_builder::{
    Library, LibraryCompilationContext, LibraryDependencies, LibraryLocation, LibraryOptions,
    LibraryTarget, PathLocation,
};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MockLibrary {
    location: LibraryLocation,
    options: LibraryOptions,
    compiled_library: PathBuf,
}

impl MockLibrary {
    fn new(source_directory: PathBuf, compiled_library: PathBuf) -> Self {
        let mut options = LibraryOptions::default();
        options.be_static(true);

        Self {
            location: LibraryLocation::Path(PathLocation::new(source_directory)),
            options,
            compiled_library,
        }
    }

    fn output_directory(&self, context: &LibraryCompilationContext) -> PathBuf {
        context.build_root().join(context.profile())
    }
}

#[typetag::serde]
impl Library for MockLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        "fake"
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
        let compiled_library = context.build_root().join(&self.compiled_library);
        fs::create_dir_all(compiled_library.parent().unwrap())?;
        fs::write(compiled_library, b"fake library")?;
        Ok(())
    }

    fn compiled_library_directories(&self, context: &LibraryCompilationContext) -> Vec<PathBuf> {
        vec![self.output_directory(context)]
    }

    fn ensure_requirements(&self, _context: &LibraryCompilationContext) {}

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
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PrebuiltLibrary {
    location: LibraryLocation,
    options: LibraryOptions,
    prebuilt_library: PathBuf,
}

impl PrebuiltLibrary {
    fn new(source_directory: PathBuf, prebuilt_library: PathBuf) -> Self {
        Self {
            location: LibraryLocation::Path(PathLocation::new(source_directory)),
            options: LibraryOptions::default(),
            prebuilt_library,
        }
    }
}

#[typetag::serde]
impl Library for PrebuiltLibrary {
    fn location(&self) -> &LibraryLocation {
        &self.location
    }

    fn name(&self) -> &str {
        "prebuilt_fake"
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

    fn retrieve_prebuilt_library(&self, context: &LibraryCompilationContext) -> Option<PathBuf> {
        Some(context.build_root().join(&self.prebuilt_library))
    }

    fn force_compile(&self, _context: &LibraryCompilationContext) -> Result<(), Box<dyn Error>> {
        panic!("prebuilt library should not be compiled")
    }

    fn compiled_library_directories(&self, _context: &LibraryCompilationContext) -> Vec<PathBuf> {
        vec![]
    }

    fn ensure_requirements(&self, _context: &LibraryCompilationContext) {}

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
        Box::new(self.clone())
    }
}

#[test]
fn compile_fake_library_and_return_compiled_path() -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    let expected_compiled_library = PathBuf::from("release/fake.lib");
    #[cfg(not(windows))]
    let expected_compiled_library = PathBuf::from("release/libfake.a");

    let test_root = std::env::temp_dir().join(format!(
        "shared-library-builder-test-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let source_root = test_root.join("src");
    let build_root = test_root.join("build");
    fs::create_dir_all(&source_root)?;
    fs::create_dir_all(&build_root)?;

    let context = LibraryCompilationContext::new(
        &source_root,
        &build_root,
        LibraryTarget::for_current_platform(),
        false,
    );
    let library = MockLibrary::new(source_root, expected_compiled_library.clone());

    let expected_path = context.build_root().join(expected_compiled_library);
    let compiled_path = library.compile(&context)?;

    assert_eq!(compiled_path, expected_path);
    assert_eq!(library.compiled_library(&context), expected_path);

    fs::remove_dir_all(test_root)?;
    Ok(())
}

#[test]
fn return_exported_shared_library_path_when_prebuilt_library_is_retrieved(
) -> Result<(), Box<dyn Error>> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-apple-darwin/release/libprebuilt_fake.dylib");

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let expected_exported_library =
        PathBuf::from("x86_64-apple-darwin/release/libprebuilt_fake.dylib");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-unknown-linux-gnu/release/libprebuilt_fake.so");

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let expected_exported_library =
        PathBuf::from("x86_64-unknown-linux-gnu/release/libprebuilt_fake.so");

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-pc-windows-msvc/release/prebuilt_fake.dll");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let expected_exported_library =
        PathBuf::from("x86_64-pc-windows-msvc/release/prebuilt_fake.dll");

    let test_root = std::env::temp_dir().join(format!(
        "shared-library-builder-prebuilt-test-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let source_root = test_root.join("src");
    let build_root = test_root.join("build");
    fs::create_dir_all(&source_root)?;
    fs::create_dir_all(&build_root)?;

    let context = LibraryCompilationContext::new(
        &source_root,
        &build_root,
        LibraryTarget::for_current_platform(),
        false,
    );
    let library = PrebuiltLibrary::new(source_root, expected_exported_library.clone());
    let expected_path = context.build_root().join(expected_exported_library);
    fs::create_dir_all(expected_path.parent().unwrap())?;
    fs::write(&expected_path, b"prebuilt library")?;

    let compiled_path = library.compile(&context)?;

    assert_eq!(library.exported_library_path(&context), expected_path);
    assert_eq!(compiled_path, expected_path);

    fs::remove_dir_all(test_root)?;
    Ok(())
}
