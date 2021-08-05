use shared_library_builder::{clipboard, Library, LibraryCompilationContext};
use std::error::Error;

#[test]
pub fn shared_release() -> Result<(), Box<dyn Error>> {
    let mut lib = clipboard();
    lib.be_shared();

    let root = std::path::PathBuf::from("target/tests/clipboard");
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
