use shared_library_builder::{CairoLibrary, Library, LibraryCompilationContext};
use std::error::Error;
use tempdir::TempDir;

#[test]
pub fn shared_release() -> Result<(), Box<dyn Error>> {
    let mut lib = CairoLibrary::new();
    lib.be_shared();

    let root = TempDir::new("build")?;
    let context = LibraryCompilationContext::new_release(&root);

    assert_eq!(
        lib.native_library_prefix(&context),
        context.build_root().join("cairo")
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
