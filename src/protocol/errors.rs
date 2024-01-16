use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

pub enum ErrorKind {
    Timeout, // Indicates that the requested operation could not be completed within a timeout.
    NodeNotFound, // Thrown when a client sends an RPC request to a node which does not exist.
    NotSupported, // Use this error to indicate that a requested operation is not supported by the current implementation. Helpful for stubbing out APIs during development.
    TemporarilyUnavailable, // Indicates that the operation definitely cannot be performed at this time--perhaps because the server is in a read-only state, has not yet been initialized, believes its peers to be down, and so on. Do not use this error for indeterminate cases, when the operation may actually have taken place.
    MalformedRequest, // The client's request did not conform to the server's expectations, and could not possibly have been processed.
    Crash, // Indicates that some kind of general, indefinite error occurred. Use this as a catch-all for errors you can't otherwise categorize, or as a starting point for your error handler: it's safe to return internal-error for every problem by default, then add special cases for more specific errors later.
    Abort, // Indicates that some kind of general, definite error occurred. Use this as a catch-all for errors you can't otherwise categorize, when you specifically know that the requested operation has not taken place. For instance, you might encounter an indefinite failure during the prepare phase of a transaction: since you haven't started the commit process yet, the transaction can't have taken place. It's therefore safe to return a definite abort to the client.
    KeyDoesNotExist, // The client requested an operation on a key which does not exist (assuming the operation should not automatically create missing keys).
    KeyAlreadyExists, // The client requested the creation of a key which already exists, and the server will not overwrite it.
    PreconditionFailed, // The requested operation expected some conditions to hold, and those conditions were not met. For instance, a compare-and-set operation might assert that the value of a key is currently 5; if the value is 3, the server would return precondition-failed.
    TxnConflict, // The requested transaction has been aborted because of a conflict with another transaction. Servers need not return this error on every conflict: they may choose to retry automatically instead.
}

impl From<ErrorKind> for usize {
    fn from(kind: ErrorKind) -> usize {
        match kind {
            ErrorKind::Timeout => 0,
            ErrorKind::NodeNotFound => 1,
            ErrorKind::NotSupported => 10,
            ErrorKind::TemporarilyUnavailable => 11,
            ErrorKind::MalformedRequest => 12,
            ErrorKind::Crash => 13,
            ErrorKind::Abort => 14,
            ErrorKind::KeyDoesNotExist => 20,
            ErrorKind::KeyAlreadyExists => 21,
            ErrorKind::PreconditionFailed => 22,
            ErrorKind::TxnConflict => 30,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorMessage {
    code: usize,
    text: String,
    #[serde(skip_serializing, skip_deserializing)]
    source: Option<Box<dyn Error + 'static>>,
}

impl ErrorMessage {
    pub fn new(error: ErrorKind, text: &str) -> ErrorMessage {
        ErrorMessage {
            code: error.into(),
            text: text.to_string(),
            source: None,
        }
    }

    pub fn with_source(self, source: impl Error + 'static) -> ErrorMessage {
        ErrorMessage {
            source: Some(Box::new(source)),
            ..self
        }
    }
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.text)?;
        if let Some(source) = &self.source {
            write!(f, "\nSource: {}", source)
        } else {
            Ok(())
        }
    }
}

impl Error for ErrorMessage {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind as IOErrorKind};

    #[test]
    fn test_display() {
        let err = ErrorMessage::new(ErrorKind::Crash, "something went wrong");
        assert_eq!("[13] something went wrong", format!("{}", err));

        let err = ErrorMessage::new(ErrorKind::Crash, "something went wrong")
            .with_source(Error::new(IOErrorKind::Other, "source error"));
        assert_eq!(
            "[13] something went wrong\nSource: source error",
            format!("{}", err)
        );
    }
}
