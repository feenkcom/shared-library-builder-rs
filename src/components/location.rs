use crate::LibraryCompilationContext;
use downloader::{Download, Downloader};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use url::Url;
use user_error::UserFacingError;

#[derive(Debug, Clone)]
pub enum LibraryLocation {
    Git(GitLocation),
    Path(PathLocation),
    Tar(TarUrlLocation),
    Zip(ZipUrlLocation),
    Multiple(Vec<LibraryLocation>),
}

impl LibraryLocation {
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
pub struct GitLocation {
    repository: Url,
    version: GitVersion,
    directory: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum GitVersion {
    Tag(String),
    Commit(String),
    Branch(String),
    Latest,
}

impl GitLocation {
    pub fn new(repository: &str) -> Self {
        Self {
            repository: Url::parse(repository).unwrap(),
            version: GitVersion::Latest,
            directory: None,
        }
    }

    pub fn commit(self, commit: impl Into<String>) -> Self {
        Self {
            repository: self.repository,
            version: GitVersion::Commit(commit.into()),
            directory: self.directory,
        }
    }

    pub fn branch(self, branch: impl Into<String>) -> Self {
        Self {
            repository: self.repository,
            version: GitVersion::Branch(branch.into()),
            directory: self.directory,
        }
    }

    pub fn tag(self, tag: impl Into<String>) -> Self {
        Self {
            repository: self.repository,
            version: GitVersion::Tag(tag.into()),
            directory: self.directory,
        }
    }

    pub fn directory(self, directory: impl Into<PathBuf>) -> Self {
        Self {
            repository: self.repository,
            version: self.version,
            directory: Some(directory.into()),
        }
    }

    fn ensure_sources(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        let source_directory = match self.directory {
            None => context.sources_root().join(default_source_directory),
            Some(ref custom_directory) => context.sources_root().join(custom_directory),
        };

        if !source_directory.exists() {
            let result = Command::new("git")
                .arg("clone")
                .arg(self.repository.to_string())
                .arg(&source_directory)
                .status()
                .unwrap();

            if !result.success() {
                return Err(Box::new(
                    UserFacingError::new("Failed to build project")
                        .reason(format!("Could not clone {}", &self.repository))
                        .help(
                            "Make sure the configuration is correct and the git repository exists",
                        ),
                ));
            }
        }

        Command::new("git")
            .current_dir(&source_directory)
            .arg("clean")
            .arg("-fdx")
            .status()
            .unwrap();

        Command::new("git")
            .current_dir(&source_directory)
            .arg("fetch")
            .arg("--all")
            .arg("--tags")
            .status()
            .unwrap();

        let status = match &self.version {
            GitVersion::Tag(tag) => Command::new("git")
                .current_dir(&source_directory)
                .arg("checkout")
                .arg(format!("tags/{}", tag))
                .status()
                .unwrap(),
            GitVersion::Commit(commit) => Command::new("git")
                .current_dir(&source_directory)
                .arg("checkout")
                .arg(commit)
                .status()
                .unwrap(),
            GitVersion::Branch(branch) => Command::new("git")
                .current_dir(&source_directory)
                .arg("checkout")
                .arg(branch)
                .status()
                .unwrap(),
            GitVersion::Latest => Command::new("git")
                .current_dir(&source_directory)
                .arg("pull")
                .status()
                .unwrap(),
        };

        if !status.success() {
            return Err(Box::new(
                UserFacingError::new("Failed to build project")
                    .reason(format!(
                        "Could not checkout {:?} of {:?}",
                        &self.version, &self.repository
                    ))
                    .help("Make sure the configuration is correct and the git repository exists"),
            ));
        }

        Ok(())
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
