use crate::{Library, LibraryCompilationContext};
use downloader::{Download, Downloader};
use std::error::Error;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;
use user_error::UserFacingError;

#[derive(Debug, Clone)]
pub struct GitLocation {
    repository: GitRepository,
    version: GitVersion,
    directory: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum GitRepository {
    GitHub(String, String),
    GitLab(String, String),
}

#[derive(Debug, Clone)]
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
}

impl ToString for GitRepository {
    fn to_string(&self) -> String {
        match self {
            GitRepository::GitHub(owner, repo) => {
                format!("https://github.com/{}/{}.git", owner, repo)
            }

            GitRepository::GitLab(owner, repo) => {
                format!("https://gitlab.com/{}/{}", owner, repo)
            }
        }
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

    pub fn directory(self, directory: impl Into<PathBuf>) -> Self {
        Self {
            repository: self.repository,
            version: self.version,
            directory: Some(directory.into()),
        }
    }

    pub(crate) fn ensure_sources(
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

                    if !build_directory.exists() {
                        create_dir_all(&build_directory).unwrap();
                    }

                    let mut downloader = Downloader::builder()
                        .download_folder(&build_directory)
                        .build()
                        .unwrap();

                    let url = format!(
                        "https://github.com/{}/{}/releases/download/{}/libGlutin-{}.dylib",
                        owner,
                        repo,
                        tag,
                        library.compiled_library_name().file_name(&format!(
                            "{}-{}",
                            library.name(),
                            context.target().to_string()
                        ))
                    );

                    let to_download = Download::new(&url);

                    let mut result = match downloader.download(&[to_download]) {
                        Ok(result) => result,
                        Err(_) => return None,
                    };
                    let download_result = result.remove(0).unwrap();

                    let downloaded_file_name = download_result.file_name;
                    let proper_file_name = downloaded_file_name
                        .with_file_name(library.compiled_library_name().file_name(library.name()));

                    std::fs::rename(downloaded_file_name, &proper_file_name).unwrap();

                    Some(proper_file_name)
                }
                _ => None,
            },
            _ => None,
        }
    }
}
