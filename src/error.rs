// Custom error types will go here
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatError {
    // Define specific error variants later
    #[error("An unspecified error occurred")]
    GenericError,
} 