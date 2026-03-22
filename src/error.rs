use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Windows(windows::core::Error),
    AlreadyRunning,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Windows(e) => write!(f, "Windows error: {}", e),
            AppError::AlreadyRunning => write!(f, "Another instance is already running"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<windows::core::Error> for AppError {
    fn from(e: windows::core::Error) -> Self {
        AppError::Windows(e)
    }
}
