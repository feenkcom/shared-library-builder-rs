use shared_library_builder::{git, Library, LibraryCompilationContext};
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

    assert_eq!(native_library_prefix, context.build_root().join("git2"));

    lib.compile(&context)?;

    Ok(())
}
