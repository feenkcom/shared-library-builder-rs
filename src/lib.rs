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
use std::path::Path;

pub use crate::library::{CompiledLibraryName, Library};
#[cfg(feature = "cmake-library")]
pub use cmake_library::CMakeLibrary;
pub use rust_library::RustLibrary;

use clap::Parser;

#[derive(Parser, Clone, Debug)]
struct BuildOptions {
    #[clap(long, ignore_case = true)]
    target: Option<LibraryTarget>,
}

pub fn with_target<F>(f: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(LibraryTarget) -> Result<(), Box<dyn std::error::Error>>,
{
    let options: BuildOptions = BuildOptions::parse();
    let target = options
        .target
        .unwrap_or_else(|| LibraryTarget::for_current_platform());

    f(target)?;
    Ok(())
}

pub fn build_standalone<F>(f: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(LibraryTarget) -> Result<Box<dyn Library>, Box<dyn std::error::Error>>,
{
    with_target(|target| {
        let library = f(target)?;

        let target_dir = Path::new("target");

        let src_dir = {
            let probable_sources_root = std::env::current_dir().unwrap().parent().unwrap().to_path_buf();
            let probable_context = LibraryCompilationContext::new(&probable_sources_root, "target", target, false);
            let exiting_sources = library.source_directory(&probable_context);
            
            if exiting_sources.exists() {
                probable_sources_root.to_path_buf()
            }
            else {
                target_dir.join("src")
            }
        };
        
        if !src_dir.exists() {
            std::fs::create_dir_all(src_dir.as_path())?;
        }
        let context = LibraryCompilationContext::new(src_dir, "target", target, false);
        let compiled_library = library.compile(&context)?;
        println!("Compiled {}", compiled_library.display());
        Ok(())
    })
}

pub fn build<F>(
    source_dir: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
    target: Option<LibraryTarget>,
    f: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(LibraryTarget) -> Result<Box<dyn Library>, Box<dyn std::error::Error>>,
{
    let source_dir = source_dir.as_ref();
    if !source_dir.exists() {
        std::fs::create_dir_all(source_dir)?;
    }

    let target_dir = target_dir.as_ref();
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir)?;
    }

    let target = target.unwrap_or_else(|| LibraryTarget::for_current_platform());
    let library = f(target)?;
    let context = LibraryCompilationContext::new(source_dir, target_dir, target, false);
    let compiled_library = library.compile(&context)?;
    println!("Compiled {}", compiled_library.display());
    Ok(())
}
