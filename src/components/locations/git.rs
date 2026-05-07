use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;

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
            GitRepository::GitHub(owner, repo) => github_downloader::retrieve_prebuilt_library(
                owner,
                repo,
                &self.version,
                self.directory.as_ref(),
                library,
                default_source_directory,
                context,
            ),
            _ => None,
        }
    }
}

#[cfg(feature = "downloader")]
mod github_downloader {
    use std::env;
    use std::error::Error;
    use std::fs::File;
    use std::io::copy;
    use std::path::{Path, PathBuf};

    use downloader::{Download, Downloader};
    use jsonwebtoken::EncodingKey;
    use octocrab::{
        models::{AppId, InstallationId},
        Octocrab,
    };
    use reqwest::blocking::Client;
    use secrecy::ExposeSecret;
    use serde::Deserialize;
    use user_error::UserFacingError;

    use super::GitVersion;
    use crate::{Library, LibraryCompilationContext};

    #[derive(Debug, Deserialize)]
    struct RepositoryMetadata {
        private: bool,
    }

    #[derive(Debug, Deserialize)]
    struct Release {
        assets: Vec<ReleaseAsset>,
    }

    #[derive(Debug, Deserialize)]
    struct ReleaseAsset {
        id: u64,
        name: String,
    }

    pub(super) fn retrieve_prebuilt_library(
        owner: &str,
        repo: &str,
        version: &GitVersion,
        directory: Option<&PathBuf>,
        library: Box<dyn Library>,
        default_source_directory: &Path,
        context: &LibraryCompilationContext,
    ) -> Option<PathBuf> {
        match version {
            GitVersion::Tag(tag) => {
                let build_directory = match directory {
                    None => context.build_root().join(default_source_directory),
                    Some(custom_directory) => context.build_root().join(custom_directory),
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

                let asset_name = library.compiled_library_name().file_name(
                    &format!("{}-{}", library.name(), context.target().to_string()),
                    context.target(),
                    false,
                );

                let installation_token =
                    match create_installation_token_if_configured(library.name()) {
                        Ok(token) => token,
                        Err(error) => {
                            eprintln!(
                                "Failed to create a GitHub installation token for {} due to {:?}",
                                library.name(),
                                error
                            );
                            return None;
                        }
                    };

                let is_private =
                    match repository_is_private(owner, repo, installation_token.as_deref()) {
                        Ok(is_private) => is_private,
                        Err(error) => {
                            eprintln!(
                                "Failed to detect visibility of GitHub repository {}/{} due to {:?}. Assuming public.",
                                owner,
                                repo,
                                error
                            );
                            false
                        }
                    };

                if is_private {
                    let Some(token) = installation_token else {
                        eprintln!(
                            "GitHub repository {}/{} is private, but {} credentials are not configured.",
                            owner,
                            repo,
                            app_env_var_prefix(library.name())
                        );
                        return None;
                    };

                    match download_private_release_asset(
                        owner,
                        repo,
                        tag,
                        &asset_name,
                        &binary_path,
                        &token,
                    ) {
                        Ok(()) => Some(binary_path),
                        Err(error) => {
                            eprintln!(
                                "Failed to download private GitHub release asset {} from {}/{}@{} due to {:?}",
                                asset_name,
                                owner,
                                repo,
                                tag,
                                error
                            );
                            None
                        }
                    }
                } else {
                    download_public_release_asset(
                        owner,
                        repo,
                        tag,
                        &asset_name,
                        &build_directory,
                        &binary_path,
                    )
                }
            }
            _ => None,
        }
    }

    fn download_public_release_asset(
        owner: &str,
        repo: &str,
        tag: &str,
        asset_name: &str,
        build_directory: &Path,
        binary_path: &Path,
    ) -> Option<PathBuf> {
        let mut downloader = Downloader::builder()
            .download_folder(build_directory)
            .build()
            .unwrap();

        let url =
            format!("https://github.com/{owner}/{repo}/releases/download/{tag}/{asset_name}");

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

        std::fs::rename(downloaded_file_name, binary_path).unwrap();

        Some(binary_path.to_path_buf())
    }

    fn download_private_release_asset(
        owner: &str,
        repo: &str,
        tag: &str,
        asset_name: &str,
        output_path: &Path,
        token: &str,
    ) -> Result<(), Box<dyn Error>> {
        let client = Client::new();

        let release: Release = client
            .get(format!(
                "https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}"
            ))
            .bearer_auth(token)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "shared-library-builder")
            .send()?
            .error_for_status()?
            .json()?;

        let asset = release
            .assets
            .into_iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                UserFacingError::new("Failed to retrieve prebuilt library").reason(format!(
                    "Could not find asset {:?} in release {:?} of {}/{}",
                    asset_name, tag, owner, repo
                ))
            })?;

        let mut response = client
            .get(format!(
                "https://api.github.com/repos/{owner}/{repo}/releases/assets/{}",
                asset.id
            ))
            .bearer_auth(token)
            .header("Accept", "application/octet-stream")
            .header("User-Agent", "shared-library-builder")
            .send()?
            .error_for_status()?;

        let mut file = File::create(output_path)?;

        copy(&mut response, &mut file)?;

        println!(
            "Downloaded {owner}/{repo} release {tag} asset {asset_name} to {}",
            output_path.display()
        );

        Ok(())
    }

    fn repository_is_private(
        owner: &str,
        repo: &str,
        token: Option<&str>,
    ) -> Result<bool, Box<dyn Error>> {
        let runtime = tokio::runtime::Runtime::new()?;
        let owner = owner.to_string();
        let repo = repo.to_string();
        let token = token.map(ToOwned::to_owned);

        runtime.block_on(async move {
            let octocrab = match token {
                Some(token) => Octocrab::builder().personal_token(token).build()?,
                None => Octocrab::builder().build()?,
            };

            let metadata: RepositoryMetadata = octocrab
                .get(format!("repos/{owner}/{repo}"), None::<&()>)
                .await?;

            Ok(metadata.private)
        })
    }

    fn create_installation_token_if_configured(
        library_name: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let app_id_key = app_env_var(library_name, "APP_ID");
        let installation_id_key = app_env_var(library_name, "APP_INSTALLATION_ID");
        let private_key_key = app_env_var(library_name, "APP_PRIVATE_KEY");

        let app_id = env::var(&app_id_key).ok();
        let installation_id = env::var(&installation_id_key).ok();
        let private_key_pem = env::var(&private_key_key).ok();

        if app_id.is_none() && installation_id.is_none() && private_key_pem.is_none() {
            return Ok(None);
        }

        let app_id: u64 = app_id
            .ok_or_else(|| missing_env_var_error(&app_id_key))?
            .parse()?;
        let installation_id: u64 = installation_id
            .ok_or_else(|| missing_env_var_error(&installation_id_key))?
            .parse()?;
        let private_key_pem = private_key_pem
            .ok_or_else(|| missing_env_var_error(&private_key_key))?
            .replace("\\n", "\n");

        let runtime = tokio::runtime::Runtime::new()?;
        let token = runtime.block_on(async move {
            let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;

            let app_client = Octocrab::builder().app(AppId(app_id), key).build()?;

            let (_github, token) = app_client
                .installation_and_token(InstallationId(installation_id))
                .await?;

            Ok::<_, Box<dyn Error>>(token.expose_secret().to_string())
        })?;

        Ok(Some(token))
    }

    fn app_env_var(library_name: &str, suffix: &str) -> String {
        format!("{}_{}", app_env_var_prefix(library_name), suffix)
    }

    fn app_env_var_prefix(library_name: &str) -> String {
        library_name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn missing_env_var_error(key: &str) -> UserFacingError {
        UserFacingError::new("Missing GitHub App configuration")
            .reason(format!("Environment variable {key} is not set"))
            .help(
                "Set the per-library GitHub App environment variables before retrieving private release assets",
            )
    }
}
