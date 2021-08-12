use shared_library_builder::{Library, LibraryCompilationContext, git};
use std::error::Error;

#[test]
pub fn shared_release() -> Result<(), Box<dyn Error>> {
    let mut lib = git();
    lib.be_shared();

    let root = std::path::PathBuf::from("target/tests/git2");
    if root.exists() {
        std::fs::remove_dir_all(&root)?
    }
    if !root.exists() {
        std::fs::create_dir_all(&root)?
    }
    let context = LibraryCompilationContext::new_release(&root);

    let native_library_prefix = lib.native_library_prefix(&context);

    if context.is_windows() {
        assert_eq!(native_library_prefix, context.sources_root().join("git2"));
    } else {
        assert_eq!(native_library_prefix, context.build_root().join("git2"));
    }

    lib.compile(&context)?;

    // let pkg_config_directory = lib.pkg_config_directory(&context);
    // if context.is_windows() {
    //     assert_eq!(pkg_config_directory, None);
    // } else {
    //     assert_eq!(
    //         pkg_config_directory,
    //         Some(
    //             lib.native_library_prefix(&context)
    //                 .join("lib")
    //                 .join("pkgconfig")
    //         )
    //     );
    // }
    //
    // let native_library_include_headers = lib.native_library_include_headers(&context);
    // if context.is_windows() {
    //     assert_eq!(
    //         native_library_include_headers,
    //         vec![lib.native_library_prefix(&context)]
    //     );
    // } else {
    //     assert_eq!(
    //         native_library_include_headers,
    //         vec![lib.native_library_prefix(&context).join("include")]
    //     );
    // }
    //
    // let native_library_linker_libraries = lib.native_library_linker_libraries(&context);
    // if context.is_windows() {
    //     assert_eq!(
    //         native_library_linker_libraries,
    //         vec![lib
    //             .native_library_prefix(&context)
    //             .join("cairo")
    //             .join(context.profile())]
    //     );
    // } else {
    //     assert_eq!(
    //         native_library_linker_libraries,
    //         vec![lib.native_library_prefix(&context).join("lib")]
    //     );
    // }

    Ok(())
}
