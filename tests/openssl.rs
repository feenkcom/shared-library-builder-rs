use shared_library_builder::{
    CairoLibrary, FreetypeLibrary, Library, LibraryCompilationContext, OpenSSLLibrary,
    PixmanLibrary,
};
use std::error::Error;
use std::path::PathBuf;
use tempdir::TempDir;

#[test]
#[cfg(not(target_os = "windows"))]
pub fn shared_release() -> Result<(), Box<dyn Error>> {
    let mut lib = OpenSSLLibrary::new();
    lib.be_static();

    let root = TempDir::new("build")?;

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
