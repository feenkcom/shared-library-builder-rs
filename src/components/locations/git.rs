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
    use std::env::VarError;
    use std::error::Error;
    use std::path::{Path, PathBuf};

    use downloader::{Download, Downloader};
    use feenk_download_auth_client::{
        download_release_asset_with_env_auth, EnvDownloadRequest, InstallationTokenSource,
    };
    use user_error::UserFacingError;

    use super::GitVersion;
    use crate::{Library, LibraryCompilationContext};

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

                match installation_token_source(library.name()) {
                    Ok(Some(token_source)) => match download_private_release_asset(
                        owner,
                        repo,
                        tag,
                        &asset_name,
                        &binary_path,
                        token_source,
                    ) {
                        Ok(()) => Some(binary_path),
                        Err(error) => {
                            eprintln!(
                                "Failed to download private GitHub release asset {} from {}/{}@{} due to {:?}",
                                asset_name, owner, repo, tag, error
                            );
                            None
                        }
                    },
                    Ok(None) => download_public_release_asset(
                        owner,
                        repo,
                        tag,
                        &asset_name,
                        &build_directory,
                        &binary_path,
                    ),
                    Err(error) => {
                        eprintln!(
                            "Failed to read GitHub authentication configuration for {} due to {:?}",
                            library.name(),
                            error
                        );
                        None
                    }
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
        token_source: InstallationTokenSource,
    ) -> Result<(), Box<dyn Error>> {
        let runtime = tokio::runtime::Runtime::new()?;
        let repo = format!("{owner}/{repo}");
        let tag = tag.to_string();
        let asset_name = asset_name.to_string();
        let output_path = output_path.to_path_buf();
        let output_display = output_path.display().to_string();

        let asset = runtime.block_on(async move {
            download_release_asset_with_env_auth(EnvDownloadRequest {
                token_source,
                repo,
                github_owner: None,
                tag: Some(tag),
                asset_name,
                output_path,
            })
            .await
        })?;

        println!(
            "Downloaded release asset {} to {}",
            asset.name,
            output_display
        );

        Ok(())
    }

    fn installation_token_source(
        library_name: &str,
    ) -> Result<Option<InstallationTokenSource>, Box<dyn Error>> {
        let installation_token_key = app_env_var(library_name, "INSTALLATION_TOKEN");

        if optional_env_value(&installation_token_key)?.is_some() {
            return Ok(Some(InstallationTokenSource::token_env(
                installation_token_key,
            )));
        }

        let private_key_key = app_env_var(library_name, "PRIVATE_KEY");
        let customer_id_key = app_env_var(library_name, "CUSTOMER_ID");
        let auth_server_url_key = app_env_var(library_name, "AUTH_SERVER_URL");
        let auth_server_key = app_env_var(library_name, "AUTH_SERVER");

        let private_key = optional_env_value(&private_key_key)?;
        let customer_id = optional_env_value(&customer_id_key)?;
        let auth_server_url = optional_env_value(&auth_server_url_key)?
            .or(optional_env_value(&auth_server_key)?);

        if private_key.is_none() && customer_id.is_none() && auth_server_url.is_none() {
            return Ok(None);
        }

        let Some(server_url) = auth_server_url else {
            return Err(Box::new(missing_env_var_error(&auth_server_url_key)));
        };

        if customer_id.is_none() {
            return Err(Box::new(missing_env_var_error(&customer_id_key)));
        }

        if private_key.is_none() {
            return Err(Box::new(missing_env_var_error(&private_key_key)));
        }

        Ok(Some(InstallationTokenSource::customer_env(
            server_url,
            customer_id_key,
            private_key_key,
        )))
    }

    fn optional_env_value(key: &str) -> Result<Option<String>, Box<dyn Error>> {
        match env::var(key) {
            Ok(value) if value.trim().is_empty() => Ok(None),
            Ok(value) => Ok(Some(value)),
            Err(VarError::NotPresent) => Ok(None),
            Err(error) => Err(Box::new(
                UserFacingError::new("Invalid GitHub authentication configuration")
                    .reason(format!(
                        "Environment variable {key} could not be read: {error}"
                    ))
                    .help("Set valid per-library GitHub authentication environment variables"),
            )),
        }
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
        UserFacingError::new("Missing GitHub authentication configuration")
            .reason(format!("Environment variable {key} is not set"))
            .help("Set either the per-library installation token, or the per-library customer id, private key, and auth server URL")
    }
}
