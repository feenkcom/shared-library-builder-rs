use shared_library_builder::{clipboard, Library, LibraryCompilationContext};
use std::error::Error;
use tempdir::TempDir;

#[test]
pub fn shared_release() -> Result<(), Box<dyn Error>> {
    let mut lib = clipboard();
    lib.be_shared();

    let root = TempDir::new("build")?;

    let context = LibraryCompilationContext::new_release(&root);

    lib.compile(&context)?;

    let compiled_library = lib.compiled_library(&context);
    assert_eq!(compiled_library.exists(), true);

    Ok(())
}
