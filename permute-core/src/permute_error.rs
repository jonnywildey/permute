use biquad::Errors as FilterErrors;
use hound::Error as HoundError;
use std::fmt::Display;
use std::io;
use std::sync::mpsc::SendError;

use crate::permute_files::PermuteUpdate;

#[derive(Debug)]
pub enum PermuteError {
    SendError(SendError<PermuteUpdate>),
    Hound(HoundError),
    IO(io::Error),
    Filter(FilterErrors),
}

impl From<HoundError> for PermuteError {
    fn from(error: HoundError) -> Self {
        PermuteError::Hound(error)
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

impl Display for PermuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
