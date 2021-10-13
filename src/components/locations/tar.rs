use crate::LibraryCompilationContext;
use downloader::{Download, Downloader};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarUrlLocation {
    url: String,
    archive: TarArchive,
    sources: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub(crate) fn sources_directory(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> PathBuf {
        context.sources_root().join(default_source_directory)
    }

    pub(crate) fn ensure_sources(
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
