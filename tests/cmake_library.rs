use shared_library_builder::{
    CMakeLibrary, CompiledLibraryName, GitLocation, Library, LibraryLocation,
};
use std::error::Error;

fn libgit2_library() -> CMakeLibrary {
    let libssh2 = CMakeLibrary::new(
        "ssh2",
        LibraryLocation::Git(GitLocation::github("libssh2", "libssh2").tag("libssh2-1.9.0")),
    )
    .define_common("CRYPTO_BACKEND", "OpenSSL");

    let libgit2 = CMakeLibrary::new(
        "git2",
        LibraryLocation::Git(GitLocation::github("libgit2", "libgit2").branch("v1.1.1")),
    )
    .compiled_name(CompiledLibraryName::Matching("git2".to_string()))
    .define_common("BUILD_CLAR", "OFF")
    .define_common("REGEX_BACKEND", "builtin")
    .define_common("USE_BUNDLED_ZLIB", "ON")
    .depends(Box::new(libssh2))
    .with_release_location(Some(LibraryLocation::Git(
        GitLocation::github("libgit2", "libgit2").tag("v1.1.1"),
    )));

    libgit2
}

#[test]
pub fn serde_serialize() -> Result<(), Box<dyn Error>> {
    let libgit2 = libgit2_library();

    let a_library = &libgit2 as &dyn Library;

    let json = serde_json::to_string_pretty(&a_library)?;
    println!("{}", json);

    let lib: Box<dyn Library> = serde_json::from_str(&json)?;
    println!("{:?}", &lib);

    Ok(())
}
