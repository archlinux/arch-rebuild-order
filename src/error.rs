use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RebuildOrderError {
    /// Given package is not present in database
    #[error("package not found")]
    PackageNotFound,

    /// Pacman database failed to initialize
    #[error("could not initialize pacman db: `{0}`")]
    PacmanDbInit(#[from] alpm::Error),

    /// Writing dotfile failed
    #[error("could not write to file: `{0}`")]
    DotfileError(#[from] io::Error),

    /// Unknown cases
    #[error("unknown error")]
    Unknown,
}
