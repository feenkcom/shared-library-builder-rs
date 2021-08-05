use shared_library_builder::{Library, LibraryCompilationContext, ZLibLibrary};
use std::error::Error;
use tempdir::TempDir;

#[test]
pub fn static_release() -> Result<(), Box<dyn Error>> {
    let mut lib = ZLibLibrary::default();
    lib.be_static();

    let root = TempDir::new("build")?;

    let context = LibraryCompilationContext::new_release(&root);

    assert_eq!(
        lib.native_library_prefix(&context),
        context.build_root().join("zlib")
    );
    lib.compile(&context)?;

    assert_eq!(
        lib.pkg_config_directory(&context),
        Some(
            lib.native_library_prefix(&context)
                .join("share")
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
