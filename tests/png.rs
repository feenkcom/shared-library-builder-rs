use shared_library_builder::{Library, LibraryCompilationContext, PngLibrary};
use std::error::Error;

#[test]
pub fn static_release() -> Result<(), Box<dyn Error>> {
    let mut lib = PngLibrary::default();
    lib.be_static();

    let root = std::path::PathBuf::from("target/tests/png");
    if root.exists() {
        std::fs::remove_dir_all(&root)?
    }
    if !root.exists() {
        std::fs::create_dir_all(&root)?
    }

    let context = LibraryCompilationContext::new_release(&root);

    assert_eq!(
        lib.native_library_prefix(&context),
        context.build_root().join("png")
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
