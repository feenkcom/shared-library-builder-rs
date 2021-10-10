extern crate fs_extra;
extern crate to_absolute;
#[cfg(feature = "cmake-library")]
mod cmake_library;
mod components;
mod library;
mod rust_library;

pub use components::*;

pub use crate::library::{CompiledLibraryName, Library};
#[cfg(feature = "cmake-library")]
pub use cmake_library::CMakeLibrary;
pub use rust_library::RustLibrary;
