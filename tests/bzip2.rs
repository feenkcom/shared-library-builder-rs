use shared_library_builder::{BZip2Library, Library, LibraryCompilationContext, NativeLibrary};
use std::error::Error;
use tempdir::TempDir;

#[test]
pub fn static_release() -> Result<(), Box<dyn Error>> {
    let mut lib = BZip2Library::default();
    lib.be_static();

    let root = TempDir::new("build")?;

    let context = LibraryCompilationContext::new_release(&root);

    assert_eq!(
        lib.native_library_prefix(&context),
        context.build_root().join("bzip2")
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
