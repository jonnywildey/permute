use biquad::Errors as FilterErrors;
use sndfile::SndFileError;
use std::fmt::Display;
use std::io;
use crossbeam_channel::SendError;
use audio_info::AudioFileError;

use crate::permute_files::PermuteUpdate;

#[derive(Debug)]
pub enum PermuteError {
    SendError(SendError<PermuteUpdate>),
    Snd(SndFileError),
    IO(io::Error),
    Filter(FilterErrors),
    Unknown(()),
}

impl From<SndFileError> for PermuteError {
    fn from(error: SndFileError) -> Self {
        PermuteError::Snd(error)
    }
}

impl From<SendError<PermuteUpdate>> for PermuteError {
    fn from(error: SendError<PermuteUpdate>) -> Self {
        PermuteError::SendError(error)
    }
}

impl From<io::Error> for PermuteError {
    fn from(error: io::Error) -> Self {
        PermuteError::IO(error)
    }
}

impl From<FilterErrors> for PermuteError {
    fn from(error: FilterErrors) -> Self {
        PermuteError::Filter(error)
    }
}

impl From<()> for PermuteError {
    fn from(error: ()) -> Self {
        PermuteError::Unknown(error)
    }
}

impl From<AudioFileError> for PermuteError {
    fn from(error: AudioFileError) -> Self {
        PermuteError::IO(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
    }
}

impl Display for PermuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
