use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug, PartialEq)]
pub struct ChecksumError;

impl fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Checksum missmatch")
    }
}

impl Error for ChecksumError {
    fn description(&self) -> &str {
        "Checksum missmatch"
    }
}

#[derive(Debug, PartialEq)]
pub struct UnknownProgrammer;

impl fmt::Display for UnknownProgrammer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown programmer")
    }
}

impl Error for UnknownProgrammer {
    fn description(&self) -> &str {
        "Unknown programmer"
    }
}

#[derive(Debug)]
pub struct SequenceError;

impl fmt::Display for SequenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sequence number missmatch")
    }
}

impl Error for SequenceError {
    fn description(&self) -> &str {
        "Sequence number missmatch"
    }
}

#[derive(Debug)]
pub struct StatusError;

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Status number missmatch")
    }
}

impl Error for StatusError {
    fn description(&self) -> &str {
        "Status number missmatch"
    }
}

#[derive(Debug)]
pub struct AnswerIdError;

impl fmt::Display for AnswerIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnswerId number missmatch")
    }
}

impl Error for AnswerIdError {
    fn description(&self) -> &str {
        "AnswerId number missmatch"
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    AnswerIdError,
    StatusError,
    SequenceError,
    ChecksumError,
    Io(io::Error),
    FromUtf8Error,
    UnknownProgrammer,
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for ErrorKind {
    fn from(_: std::string::FromUtf8Error) -> ErrorKind {
        ErrorKind::FromUtf8Error
    }
}

impl From<ChecksumError> for ErrorKind {
    fn from(_: ChecksumError) -> ErrorKind {
        ErrorKind::ChecksumError
    }
}

impl From<UnknownProgrammer> for ErrorKind {
    fn from(_: UnknownProgrammer) -> ErrorKind {
        ErrorKind::UnknownProgrammer
    }
}
