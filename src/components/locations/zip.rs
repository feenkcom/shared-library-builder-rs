use crate::LibraryCompilationContext;
use downloader::{Download, Downloader};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};

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
