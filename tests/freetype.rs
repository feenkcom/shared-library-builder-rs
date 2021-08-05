use shared_library_builder::{FreetypeLibrary, Library, LibraryCompilationContext, PixmanLibrary};
use std::error::Error;
use std::path::PathBuf;
use tempdir::TempDir;

#[test]
pub fn static_release() -> Result<(), Box<dyn Error>> {
    let mut lib = FreetypeLibrary::default();
    lib.be_static();

    let root = TempDir::new("build")?;

    let context = LibraryCompilationContext::new_release(&root);

    assert_eq!(
        lib.native_library_prefix(&context),
        context.build_root().join("freetype")
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

#[test]
pub fn shared_release() -> Result<(), Box<dyn Error>> {
    let mut lib = FreetypeLibrary::default();
    lib.be_shared();

    let root = TempDir::new("build")?;

    let context = LibraryCompilationContext::new_release(&root);

    lib.compile(&context)?;

    let compiled_library = lib.compiled_library(&context);
    assert_eq!(compiled_library.exists(), true);

    Ok(())
}
