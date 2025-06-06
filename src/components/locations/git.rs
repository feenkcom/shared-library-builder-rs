use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(feature = "downloader")]
use downloader::{Download, Downloader};
use serde::{Deserialize, Serialize};
use url::Url;
use user_error::UserFacingError;

use crate::{Library, LibraryCompilationContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLocation {
    repository: GitRepository,
    version: GitVersion,
    directory: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitRepository {
    GitHub(String, String),
    GitLab(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitVersion {
    Tag(String),
    Commit(String),
    Branch(String),
    Latest,
}

impl GitRepository {
    pub fn github(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self::GitHub(owner.into(), repo.into())
    }

    pub fn gitlab(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self::GitLab(owner.into(), repo.into())
    }

    pub fn as_url(&self) -> Url {
        Url::parse(self.to_string().as_str()).unwrap()
    }

    pub fn repository_name(&self) -> &str {
        match self {
            GitRepository::GitHub(_, name) => name.as_str(),
            GitRepository::GitLab(_, name) => name.as_str(),
        }
    }
}

impl Display for GitRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GitRepository::GitHub(owner, repo) => {
                format!("https://github.com/{}/{}.git", owner, repo)
            }

            GitRepository::GitLab(owner, repo) => {
                format!("https://gitlab.com/{}/{}", owner, repo)
            }
        };
        write!(f, "{}", str)
    }
}

impl GitLocation {
    pub fn new(repository: GitRepository) -> Self {
        Self {
            repository,
            version: GitVersion::Latest,
            directory: None,
        }
    }

    pub fn github(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self::new(GitRepository::github(owner, repo))
    }

    pub fn gitlab(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self::new(GitRepository::gitlab(owner, repo))
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

    pub fn tag_or_latest(self, tag: Option<impl Into<String>>) -> Self {
        let version = tag
            .map(|tag| GitVersion::Tag(tag.into()))
            .unwrap_or_else(|| GitVersion::Latest);
        Self {
            repository: self.repository,
            version,
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

    pub(crate) fn sources_directory(
        &self,
        _default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> PathBuf {
        match self.directory {
            None => context
                .sources_root()
                .join(self.repository.repository_name()),
            Some(ref custom_directory) => context.sources_root().join(custom_directory),
        }
    }

    pub(crate) fn ensure_sources(
        &self,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Result<(), Box<dyn Error>> {
        let source_directory = self.sources_directory(default_source_directory, context);

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
                        .reason(format!("Could not clone {}", &self.repository.as_url()))
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

    #[cfg(not(feature = "downloader"))]
    pub(crate) fn retrieve_prebuilt_library(
        &self,
        _library: Box<dyn Library>,
        _default_source_directory: &Path,
        _context: &LibraryCompilationContext,
    ) -> Option<PathBuf> {
        None
    }

    #[cfg(feature = "downloader")]
    pub(crate) fn retrieve_prebuilt_library(
        &self,
        library: Box<dyn Library>,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Option<PathBuf> {
        match &self.repository {
            GitRepository::GitHub(owner, repo) => match &self.version {
                GitVersion::Tag(tag) => {
                    let build_directory = match self.directory {
                        None => context.build_root().join(default_source_directory),
                        Some(ref custom_directory) => context.build_root().join(custom_directory),
                    };

                    let binary_name = library.compiled_library_name().file_name(
                        library.name(),
                        context.target(),
                        false,
                    );
                    let binary_path = build_directory.join(binary_name);

                    if binary_path.exists() {
                        println!("{} already exists.", binary_path.display());
                        return Some(binary_path);
                    }

                    if !build_directory.exists() {
                        std::fs::create_dir_all(&build_directory).unwrap();
                    }

                    let mut downloader = Downloader::builder()
                        .download_folder(&build_directory)
                        .build()
                        .unwrap();

                    let url = format!(
                        "https://github.com/{}/{}/releases/download/{}/{}",
                        owner,
                        repo,
                        tag,
                        library.compiled_library_name().file_name(
                            &format!("{}-{}", library.name(), context.target().to_string()),
                            context.target(),
                            false
                        )
                    );

                    let to_download = Download::new(&url);

                    let mut result = match downloader.download(&[to_download]) {
                        Ok(result) => result,
                        Err(error) => {
                            eprintln!("Failed to download {} due to {:?}", &url, error);
                            return None;
                        }
                    };
                    let download_result = match result.remove(0) {
                        Ok(result) => result,
                        Err(error) => {
                            eprintln!("Failed to download {} due to {:?}", &url, error);
                            return None;
                        }
                    };

                    let downloaded_file_name = download_result.file_name;
                    let proper_file_name = downloaded_file_name.with_file_name(
                        library.compiled_library_name().file_name(
                            library.name(),
                            context.target(),
                            false,
                        ),
                    );

                    std::fs::rename(downloaded_file_name, &proper_file_name).unwrap();

                    Some(proper_file_name)
                }
                _ => None,
            },
            _ => None,
        }
    }
}
