use shared_library_builder::{crypto, Library, LibraryCompilationContext, OpenSSLLibrary};
use std::error::Error;

#[test]
pub fn shared_release() -> Result<(), Box<dyn Error>> {
    let mut lib = crypto();

    let root = std::path::PathBuf::from("target/tests/openssl-shared");
    if root.exists() {
        std::fs::remove_dir_all(&root)?
    }
    if !root.exists() {
        std::fs::create_dir_all(&root)?
    }

    let context = LibraryCompilationContext::new_release(&root);

    assert_eq!(
        lib.native_library_prefix(&context),
        context.build_root().join("crypto").join("build")
    );
    lib.compile(&context)?;

    assert_eq!(
        lib.pkg_config_directory(&context),
        Some(
            lib.native_library_prefix(&context)
                .join("lib")
                .join("pkgconfig")
        )
    );

    assert_eq!(
        lib.native_library_include_headers(&context),
        vec![lib.native_library_prefix(&context).join("include")]
    );

    assert_eq!(
        lib.native_library_linker_libraries(&context),
        vec![lib.native_library_prefix(&context).join("lib")]
    );

    assert_eq!(lib.compiled_library_directories(&context).len(), 1);
    assert_eq!(lib.compiled_library(&context).exists(), true);

    Ok(())
}

#[test]
pub fn static_release() -> Result<(), Box<dyn Error>> {
    let mut lib = OpenSSLLibrary::new();
    lib.be_static();

    let root = std::path::PathBuf::from("target/tests/openssl-static");
    if root.exists() {
        std::fs::remove_dir_all(&root)?
    }
    if !root.exists() {
        std::fs::create_dir_all(&root)?
    }

    let context = LibraryCompilationContext::new_release(&root);

    assert_eq!(
        lib.native_library_prefix(&context),
        context.build_root().join("openssl").join("build")
    );
    lib.compile(&context)?;

    assert_eq!(
        lib.pkg_config_directory(&context),
        Some(
            lib.native_library_prefix(&context)
                .join("lib")
                .join("pkgconfig")
        )
    );

    assert_eq!(
        lib.native_library_include_headers(&context),
        vec![lib.native_library_prefix(&context).join("include")]
    );

    assert_eq!(
        lib.native_library_linker_libraries(&context),
        vec![lib.native_library_prefix(&context).join("lib")]
    );

    Ok(())
}
