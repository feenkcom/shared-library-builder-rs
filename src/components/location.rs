use crate::{Library, LibraryCompilationContext, LibraryGitLocation};
use downloader::{Download, Downloader};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;
use user_error::UserFacingError;

#[derive(Debug, Clone)]
pub enum LibraryLocation {
    Git(LibraryGitLocation),
    Path(PathLocation),
    Tar(TarUrlLocation),
    Zip(ZipUrlLocation),
    Multiple(Vec<LibraryLocation>),
}

impl LibraryLocation {
    pub fn github(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self::Git(LibraryGitLocation::github(owner, repo))
    }

    /// Try to retrieve a prebuilt library for the current target and return a path to the file
    pub fn retrieve_prebuilt_library(
        &self,
        library: Box<dyn Library>,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Option<PathBuf> {
        // Static libraries must be compiled from sources
        if library.is_static() {
            return None;
        }

        match self {
            LibraryLocation::Git(git_location) => {
                git_location.retrieve_prebuilt_library(library, default_source_directory, context)
            }
            _ => None,
        }
    }

    pub fn ensure_sources(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            LibraryLocation::Git(git_location) => {
                git_location.ensure_sources(default_source_directory, context)
            }
            LibraryLocation::Path(path_location) => {
                path_location.ensure_sources(default_source_directory, context)
            }
            LibraryLocation::Multiple(locations) => {
                for location in locations {
                    location.ensure_sources(default_source_directory, context)?;
                }

                Ok(())
            }
            LibraryLocation::Tar(tar_location) => {
                tar_location.ensure_sources(default_source_directory, context)
            }
            LibraryLocation::Zip(zip_location) => {
                zip_location.ensure_sources(default_source_directory, context)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathLocation {
    path: PathBuf,
}

impl PathLocation {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn ensure_sources(
        &self,
        default_source_directory: &Path,
        _context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        if !default_source_directory.exists() {
            return Err(Box::new(
                UserFacingError::new("Failed to build project")
                    .reason(format!(
                        "{} sources directory does not exist",
                        self.path.display()
                    ))
                    .help("Make sure the configuration is correct and the sources exist"),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TarUrlLocation {
    url: String,
    archive: TarArchive,
    sources: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum TarArchive {
    Gz,
    Xz,
}

impl TarUrlLocation {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            archive: TarArchive::Gz,
            sources: None,
        }
    }

    pub fn archive(self, archive: TarArchive) -> Self {
        Self {
            url: self.url,
            archive,
            sources: self.sources,
        }
    }

    pub fn sources(self, folder: impl Into<PathBuf>) -> Self {
        Self {
            url: self.url,
            archive: self.archive,
            sources: Some(folder.into()),
        }
    }

    fn ensure_sources(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        let source_directory = context.sources_root().join(default_source_directory);

        if !source_directory.exists() {
            std::fs::create_dir_all(&source_directory)?;

            let mut downloader = Downloader::builder()
                .download_folder(&source_directory)
                .build()?;

            let to_download = Download::new(&self.url);

            let mut result = downloader.download(&[to_download])?;
            let download_result = result.remove(0)?;
            let downloaded_path = download_result.file_name;

            let downloaded_tar = File::open(&downloaded_path)?;

            match self.archive {
                TarArchive::Gz => {
                    let xz = flate2::read::GzDecoder::new(downloaded_tar);
                    let mut archive = Archive::new(xz);
                    archive.unpack(&source_directory)?;
                }
                TarArchive::Xz => {
                    let xz = xz2::read::XzDecoder::new(downloaded_tar);
                    let mut archive = Archive::new(xz);
                    archive.unpack(&source_directory)?;
                }
            }

            std::fs::remove_file(&downloaded_path)?;

            if let Some(ref sources) = self.sources {
                let copy_options = fs_extra::dir::CopyOptions {
                    content_only: true,
                    ..Default::default()
                };

                fs_extra::dir::copy(
                    source_directory.join(sources),
                    &source_directory,
                    &copy_options,
                )?;

                std::fs::remove_dir_all(source_directory.join(sources))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ZipUrlLocation {
    url: String,
    sources: Option<PathBuf>,
}

impl ZipUrlLocation {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            sources: None,
        }
    }

    pub fn sources(self, folder: impl Into<PathBuf>) -> Self {
        Self {
            url: self.url,
            sources: Some(folder.into()),
        }
    }

    fn ensure_sources(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        let source_directory = context.sources_root().join(default_source_directory);

        if !source_directory.exists() {
            std::fs::create_dir_all(&source_directory)?;

            let mut downloader = Downloader::builder()
                .download_folder(&source_directory)
                .build()?;

            let to_download = Download::new(&self.url);

            let mut result = downloader.download(&[to_download])?;
            let download_result = result.remove(0)?;
            let downloaded_path = download_result.file_name;

            let downloaded_zip = File::open(&downloaded_path)?;
            let mut archive = zip::ZipArchive::new(downloaded_zip)?;
            archive.extract(&source_directory)?;

            std::fs::remove_file(&downloaded_path)?;

            if let Some(ref sources) = self.sources {
                let copy_options = fs_extra::dir::CopyOptions {
                    content_only: true,
                    ..Default::default()
                };

                fs_extra::dir::copy(
                    source_directory.join(sources),
                    &source_directory,
                    &copy_options,
                )?;

                std::fs::remove_dir_all(source_directory.join(sources))?;
            }
        }
        Ok(())
    }
}
