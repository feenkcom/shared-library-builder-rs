use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumString, Display)]
pub enum LibraryTarget {
    #[strum(serialize = "x86_64-apple-darwin")]
    X8664appleDarwin,
    #[strum(serialize = "aarch64-apple-darwin")]
    AArch64appleDarwin,
    #[strum(serialize = "x86_64-pc-windows-msvc")]
    X8664pcWindowsMsvc,
    #[strum(serialize = "aarch64-pc-windows-msvc")]
    AArch64pcWindowsMsvc,
    #[strum(serialize = "x86_64-unknown-linux-gnu")]
    X8664UnknownlinuxGNU,
}

impl LibraryTarget {
    pub fn for_current_platform() -> Self {
        if let Ok(build_string) = std::env::var("CARGO_BUILD_TARGET") {
            return Self::from_str(build_string.as_str()).unwrap();
        }
        match std::env::consts::OS {
            "linux" => match std::env::consts::ARCH {
                "x86_64" => Self::X8664UnknownlinuxGNU,
                _ => panic!("Unsupported ARCH"),
            },
            "macos" => match std::env::consts::ARCH {
                "x86_64" => Self::X8664appleDarwin,
                "aarch64" => Self::AArch64appleDarwin,
                _ => panic!("Unsupported ARCH"),
            },
            "windows" => match std::env::consts::ARCH {
                "x86_64" => Self::X8664pcWindowsMsvc,
                "aarch64" => Self::AArch64pcWindowsMsvc,
                _ => panic!("Unsupported ARCH"),
            },
            _ => panic!("Unsupported OS"),
        }
    }

    pub fn is_current(&self) -> bool {
        self.eq(&Self::for_current_platform())
    }

    pub fn is_unix(&self) -> bool {
        self.is_linux() | self.is_mac()
    }

    pub fn is_linux(&self) -> bool {
        match self {
            Self::X8664appleDarwin => false,
            Self::AArch64appleDarwin => false,
            Self::X8664pcWindowsMsvc => false,
            Self::AArch64pcWindowsMsvc => false,
            Self::X8664UnknownlinuxGNU => true,
        }
    }

    pub fn is_mac(&self) -> bool {
        match self {
            Self::X8664appleDarwin => true,
            Self::AArch64appleDarwin => true,
            Self::X8664pcWindowsMsvc => false,
            Self::AArch64pcWindowsMsvc => false,
            Self::X8664UnknownlinuxGNU => false,
        }
    }

    pub fn is_windows(&self) -> bool {
        match self {
            Self::X8664appleDarwin => false,
            Self::AArch64appleDarwin => false,
            Self::X8664pcWindowsMsvc => true,
            Self::AArch64pcWindowsMsvc => true,
            Self::X8664UnknownlinuxGNU => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(
            LibraryTarget::X8664appleDarwin.to_string(),
            "x86_64-apple-darwin".to_string()
        );
    }

    #[test]
    fn from_string() {
        let target = LibraryTarget::from_str("x86_64-apple-darwin").unwrap();
        assert_eq!(target, LibraryTarget::X8664appleDarwin);
    }
}
