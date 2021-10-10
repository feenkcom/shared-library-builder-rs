#[cfg(feature = "git-location")]
mod git;
mod path;
#[cfg(feature = "tar-location")]
mod tar;
#[cfg(feature = "zip-location")]
mod zip;

#[cfg(feature = "git-location")]
pub use self::git::{GitLocation, GitRepository, GitVersion};
pub use self::path::PathLocation;
#[cfg(feature = "tar-location")]
pub use self::tar::{TarArchive, TarUrlLocation};
#[cfg(feature = "zip-location")]
pub use self::zip::ZipUrlLocation;
