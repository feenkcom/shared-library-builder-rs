#[derive(Copy, Clone, Debug)]
pub enum LibraryTarget {
    X8664appleDarwin,
    AArch64appleDarwin,
    X8664pcWindowsMsvc,
    X8664UnknownlinuxGNU,
}

impl ToString for LibraryTarget {
    fn to_string(&self) -> String {
        (match self {
            LibraryTarget::X8664appleDarwin => "x86_64-apple-darwin",
            LibraryTarget::AArch64appleDarwin => "aarch64-apple-darwin",
            LibraryTarget::X8664pcWindowsMsvc => "x86_64-pc-windows-msvc",
            LibraryTarget::X8664UnknownlinuxGNU => "x86_64-unknown-linux-gnu",
        })
        .to_string()
    }
}

impl LibraryTarget {
    pub fn for_current_platform() -> Self {
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
                _ => panic!("Unsupported ARCH"),
            },
            _ => panic!("Unsupported OS"),
        }
    }

    pub fn is_unix(&self) -> bool {
        match self {
            Self::X8664appleDarwin => true,
            Self::AArch64appleDarwin => true,
            Self::X8664pcWindowsMsvc => false,
            Self::X8664UnknownlinuxGNU => true,
        }
    }

    pub fn is_linux(&self) -> bool {
        match self {
            Self::X8664appleDarwin => false,
            Self::AArch64appleDarwin => false,
            Self::X8664pcWindowsMsvc => false,
            Self::X8664UnknownlinuxGNU => true,
        }
    }

    pub fn is_mac(&self) -> bool {
        match self {
            Self::X8664appleDarwin => true,
            Self::AArch64appleDarwin => true,
            Self::X8664pcWindowsMsvc => false,
            Self::X8664UnknownlinuxGNU => false,
        }
    }

    pub fn is_windows(&self) -> bool {
        match self {
            Self::X8664appleDarwin => false,
            Self::AArch64appleDarwin => false,
            Self::X8664pcWindowsMsvc => true,
            Self::X8664UnknownlinuxGNU => false,
        }
    }
}
