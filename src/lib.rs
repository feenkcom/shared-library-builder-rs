extern crate fs_extra;
extern crate to_absolute;
#[cfg(feature = "cmake-library")]
mod cmake_library;
mod components;
//mod libraries;
mod library;
mod rust_library;

pub use components::*;
//pub use libraries::*;

pub use crate::library::{CompiledLibraryName, Library};
#[cfg(feature = "cmake-library")]
pub use cmake_library::CMakeLibrary;
pub use rust_library::RustLibrary;
//
// pub fn crypto() -> OpenSSLLibrary {
//     OpenSSLLibrary::new().be_crypto()
// }
//
// pub fn ssl() -> OpenSSLLibrary {
//     OpenSSLLibrary::new().be_ssl()
// }
//
// pub fn git() -> CMakeLibrary {
//     let openssl = OpenSSLLibrary::new();
//
//     let libssh2 = CMakeLibrary::new(
//         "ssh2",
//         LibraryLocation::Git(GitLocation::github("libssh2", "libssh2").tag("libssh2-1.9.0")),
//     )
//     .define_common("CRYPTO_BACKEND", "OpenSSL")
//     .depends(Box::new(openssl));
//
//     CMakeLibrary::new(
//         "git2",
//         LibraryLocation::Git(
//             GitLocation::github("syrel", "libgit2").branch("v1.1.1-windows-openssl"),
//         ),
//     )
//     .compiled_name(CompiledLibraryName::Matching("git2".to_string()))
//     .define_common("BUILD_CLAR", "OFF")
//     .define_common("REGEX_BACKEND", "builtin")
//     .define_common("USE_BUNDLED_ZLIB", "ON")
//     .depends(Box::new(libssh2))
// }
//
// pub fn sdl2() -> CMakeLibrary {
//     CMakeLibrary::new(
//         "SDL2",
//         LibraryLocation::Git(GitLocation::github("libsdl-org", "SDL").tag("release-2.0.14")),
//     )
//     .compiled_name(CompiledLibraryName::Matching("SDL2".to_string()))
// }
//
// pub fn glutin() -> RustLibrary {
//     RustLibrary::new(
//         "Glutin",
//         LibraryLocation::Git(GitLocation::github("feenkcom", "libglutin")),
//     )
// }
//
// pub fn boxer() -> RustLibrary {
//     RustLibrary::new(
//         "Boxer",
//         LibraryLocation::Git(GitLocation::github("feenkcom", "gtoolkit-boxer")),
//     )
// }
//
// pub fn skia() -> RustLibrary {
//     RustLibrary::new(
//         "Skia",
//         LibraryLocation::Git(GitLocation::github("feenkcom", "libskia")),
//     )
//     .requires("python")
// }
//
// pub fn gleam() -> RustLibrary {
//     RustLibrary::new(
//         "Gleam",
//         LibraryLocation::Git(GitLocation::github("feenkcom", "libgleam")),
//     )
// }
//
// pub fn winit() -> RustLibrary {
//     RustLibrary::new(
//         "Winit",
//         LibraryLocation::Git(GitLocation::github("feenkcom", "libwinit")),
//     )
// }
//
// pub fn clipboard() -> RustLibrary {
//     RustLibrary::new(
//         "Clipboard",
//         LibraryLocation::Git(GitLocation::github("feenkcom", "libclipboard")),
//     )
// }
