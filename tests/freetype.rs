use shared_library_builder::{FreetypeLibrary, Library, LibraryCompilationContext};
use std::error::Error;

#[test]
pub fn static_release() -> Result<(), Box<dyn Error>> {
    let mut lib = FreetypeLibrary::default();
    lib.be_static();

    let root = std::path::PathBuf::from("target/tests/freetype-static");
    if root.exists() {
        std::fs::remove_dir_all(&root)?
    }
    if !root.exists() {
        std::fs::create_dir_all(&root)?
    }

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
        vec![
            lib.native_library_prefix(&context).join("include"),
            lib.native_library_prefix(&context)
                .join("include")
                .join("freetype2")
        ]
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

    let root = std::path::PathBuf::from("target/tests/freetype-shared");
    if root.exists() {
        std::fs::remove_dir_all(&root)?
    }
    if !root.exists() {
        std::fs::create_dir_all(&root)?
    }

    let context = LibraryCompilationContext::new_release(&root);

    lib.compile(&context)?;

    let compiled_library = lib.compiled_library(&context);
    assert_eq!(compiled_library.exists(), true);

    Ok(())
}
