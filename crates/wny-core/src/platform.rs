use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
    Linux,
    Android,
    Ios,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::Macos
        } else if cfg!(target_os = "android") {
            Platform::Android
        } else if cfg!(target_os = "ios") {
            Platform::Ios
        } else {
            Platform::Linux
        }
    }

    pub fn has_filter_slot(self) -> bool {
        !matches!(self, Platform::Ios)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::Macos => "macos",
            Platform::Linux => "linux",
            Platform::Android => "android",
            Platform::Ios => "ios",
        }
    }
}

pub fn triple() -> String {
    let os = Platform::current().as_str();
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };
    // engines.lock uses darwin-* for macOS artifacts
    let os = if os == "macos" { "darwin" } else { os };
    format!("{os}-{arch}")
}
