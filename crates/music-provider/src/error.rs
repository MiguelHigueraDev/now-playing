use thiserror::Error;

pub type Result<T> = std::result::Result<T, MusicProviderError>;

#[derive(Debug, Error)]
pub enum MusicProviderError {
    #[error("failed to run AppleScript: {0}")]
    AppleScriptFailed(String),

    #[error("AppleScript returned unexpected output: {0}")]
    UnexpectedOutput(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
