use std::{error::Error, fmt};

#[derive(PartialEq, Clone, Debug)]
pub enum HttpParseError {
	MissingMethod,
	MissingRequestTarget,
	MissingHttpVersion,
	BadHeader(String),
}

impl fmt::Display for HttpParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			HttpParseError::MissingMethod => write!(f, "method not passed in request"),
			HttpParseError::MissingRequestTarget => write!(f, "request target not passed in request"),
			HttpParseError::MissingHttpVersion => write!(f, "HTTP version not passed in request"),
			HttpParseError::BadHeader(s) => write!(f, "malformed header in request: {}", s),
		}
	}
}

impl Error for HttpParseError {}
