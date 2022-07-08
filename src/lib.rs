extern crate fs_extra;
extern crate to_absolute;
#[macro_use]
extern crate strum;
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

pub fn build_standalone<F>(f: F) -> Result<(), Box<dyn std::error::Error>> where
    F: FnOnce(LibraryTarget) -> Result<Box<dyn Library>, Box<dyn std::error::Error>> {

    let target = LibraryTarget::for_current_platform();

    let library = f(target)?;

    let context = LibraryCompilationContext::new("target", "target", target, false);
    let compiled_library = library.compile(&context)?;
    println!("Compiled {}", compiled_library.display());
    Ok(())
}