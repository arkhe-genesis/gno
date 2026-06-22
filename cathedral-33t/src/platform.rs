use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
    Android,
    iOS,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_os = "ios") {
            Self::iOS
        } else {
            Self::Unknown
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Windows => "Windows",
            Self::MacOS => "macOS",
            Self::Android => "Android",
            Self::iOS => "iOS",
            Self::Unknown => "Unknown",
        }
    }

    pub fn has_gpu_acceleration(&self) -> bool {
        match self {
            Self::Linux | Self::Windows | Self::MacOS => true,
            Self::Android | Self::iOS => true,
            _ => false,
        }
    }

    pub fn default_model_path(&self) -> PathBuf {
        match self {
            Self::Android | Self::iOS => {
                PathBuf::from("assets/model.qt")
            }
            Self::Linux | Self::Windows | Self::MacOS => {
                PathBuf::from("/models/cathedral_33t.qt")
            }
            _ => PathBuf::from("model.qt"),
        }
    }
}

pub fn init_platform() {
    match Platform::current() {
        Platform::Android => {
            #[cfg(target_os = "android")]
            android_logger::init_once(
                android_logger::Config::default()
                    .with_min_level(log::Level::Info)
            );
        }
        Platform::iOS => {
            #[cfg(target_os = "ios")]
            unsafe {
            }
        }
        _ => {
            tracing_subscriber::fmt::init();
        }
    }
}
