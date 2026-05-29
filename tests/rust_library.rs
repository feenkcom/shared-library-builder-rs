use shared_library_builder::{
    Library, LibraryCompilationContext, LibraryLocation, LibraryTarget, PathLocation, RustLibrary,
};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn return_target_qualified_prebuilt_library_asset_name() -> Result<(), Box<dyn Error>> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let expected_asset_name = "librust_prebuilt-aarch64-apple-darwin.dylib";

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let expected_asset_name = "librust_prebuilt-x86_64-apple-darwin.dylib";

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let expected_asset_name = "librust_prebuilt-aarch64-unknown-linux-gnu.so";

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let expected_asset_name = "librust_prebuilt-x86_64-unknown-linux-gnu.so";

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let expected_asset_name = "rust_prebuilt-aarch64-pc-windows-msvc.dll";

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let expected_asset_name = "rust_prebuilt-x86_64-pc-windows-msvc.dll";

    let test_root = std::env::temp_dir().join(format!(
        "shared-library-builder-asset-name-test-{}-{}",
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
    let library = RustLibrary::new(
        "rust_prebuilt",
        LibraryLocation::Path(PathLocation::new(source_root.clone())),
    );

    assert_eq!(
        library.prebuilt_library_asset_name(&context),
        expected_asset_name
    );

    fs::remove_dir_all(test_root)?;
    Ok(())
}

#[test]
fn compile_rust_shared_library_and_return_compiled_path() -> Result<(), Box<dyn Error>> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let expected_compiled_library = PathBuf::from("release/librust_fake.dylib");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-apple-darwin/release/librust_fake.dylib");

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let expected_compiled_library = PathBuf::from("release/librust_fake.dylib");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let expected_exported_library = PathBuf::from("x86_64-apple-darwin/release/librust_fake.dylib");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let expected_compiled_library = PathBuf::from("release/librust_fake.so");
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-unknown-linux-gnu/release/librust_fake.so");

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let expected_compiled_library = PathBuf::from("release/librust_fake.so");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let expected_exported_library =
        PathBuf::from("x86_64-unknown-linux-gnu/release/librust_fake.so");

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let expected_compiled_library = PathBuf::from("release/rust_fake.dll");
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let expected_exported_library = PathBuf::from("aarch64-pc-windows-msvc/release/rust_fake.dll");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let expected_compiled_library = PathBuf::from("release/rust_fake.dll");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let expected_exported_library = PathBuf::from("x86_64-pc-windows-msvc/release/rust_fake.dll");

    let test_root = std::env::temp_dir().join(format!(
        "shared-library-builder-rust-test-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let source_root = test_root.join("src");
    let crate_source = source_root.join("rust_fake");
    let crate_src = crate_source.join("src");
    let build_root = test_root.join("build");
    fs::create_dir_all(&crate_src)?;
    fs::create_dir_all(&build_root)?;
    fs::write(
        crate_source.join("Cargo.toml"),
        r#"[package]
name = "rust-fake"
version = "0.1.0"
edition = "2021"

[lib]
name = "rust_fake"
crate-type = ["cdylib"]
"#,
    )?;
    fs::write(
        crate_src.join("lib.rs"),
        r#"#[no_mangle]
pub extern "C" fn rust_fake_answer() -> i32 {
    42
}
"#,
    )?;

    let context = LibraryCompilationContext::new(
        &source_root,
        &build_root,
        LibraryTarget::for_current_platform(),
        false,
    );
    let library = RustLibrary::new(
        "rust_fake",
        LibraryLocation::Path(PathLocation::new(crate_source)),
    );

    let expected_compiled_path = context.build_root().join(expected_compiled_library);
    let expected_exported_path = context.build_root().join(expected_exported_library);
    let compiled_path = library.compile(&context)?;

    assert_eq!(compiled_path, expected_exported_path);
    assert_eq!(library.compiled_library(&context), expected_compiled_path);

    fs::remove_dir_all(test_root)?;
    Ok(())
}

#[test]
fn copy_rust_prebuilt_library_to_exported_path() -> Result<(), Box<dyn Error>> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let prebuilt_library = PathBuf::from("librust_prebuilt-aarch64-apple-darwin.dylib");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-apple-darwin/release/librust_prebuilt.dylib");

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let prebuilt_library = PathBuf::from("librust_prebuilt-x86_64-apple-darwin.dylib");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let expected_exported_library =
        PathBuf::from("x86_64-apple-darwin/release/librust_prebuilt.dylib");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let prebuilt_library = PathBuf::from("librust_prebuilt-aarch64-unknown-linux-gnu.so");
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-unknown-linux-gnu/release/librust_prebuilt.so");

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let prebuilt_library = PathBuf::from("librust_prebuilt-x86_64-unknown-linux-gnu.so");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let expected_exported_library =
        PathBuf::from("x86_64-unknown-linux-gnu/release/librust_prebuilt.so");

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let prebuilt_library = PathBuf::from("rust_prebuilt-aarch64-pc-windows-msvc.dll");
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    let expected_exported_library =
        PathBuf::from("aarch64-pc-windows-msvc/release/rust_prebuilt.dll");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let prebuilt_library = PathBuf::from("rust_prebuilt-x86_64-pc-windows-msvc.dll");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let expected_exported_library =
        PathBuf::from("x86_64-pc-windows-msvc/release/rust_prebuilt.dll");

    let test_root = std::env::temp_dir().join(format!(
        "shared-library-builder-rust-prebuilt-test-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let source_root = test_root.join("src");
    let crate_source = source_root.join("rust_prebuilt");
    let prebuilt_directory = test_root.join("prebuilt");
    let build_root = test_root.join("build");
    fs::create_dir_all(&source_root)?;
    fs::create_dir_all(&prebuilt_directory)?;
    fs::create_dir_all(&build_root)?;
    fs::write(
        prebuilt_directory.join(prebuilt_library),
        b"prebuilt rust library",
    )?;

    let context = LibraryCompilationContext::new(
        &source_root,
        &build_root,
        LibraryTarget::for_current_platform(),
        false,
    );
    let library = RustLibrary::new(
        "rust_prebuilt",
        LibraryLocation::Path(
            PathLocation::new(crate_source).prebuilt_library_directory(prebuilt_directory),
        ),
    );
    let expected_path = context.build_root().join(expected_exported_library);

    let compiled_path = library.compile(&context)?;

    assert_eq!(compiled_path, expected_path);
    assert_eq!(library.exported_library_path(&context), expected_path);
    assert_eq!(fs::read(&expected_path)?, b"prebuilt rust library");

    fs::remove_dir_all(test_root)?;
    Ok(())
}
