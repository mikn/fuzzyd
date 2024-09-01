use thiserror::Error;

#[derive(Error, Debug)]
pub enum FuzzydError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to launch command: {0}")]
    LaunchError(String),
    #[error("User interrupted the operation")]
    UserInterrupt,
}