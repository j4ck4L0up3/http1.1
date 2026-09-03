use std::{error::Error, fmt};

#[derive(PartialEq, Clone, Debug)]
pub enum HttpParseError {
	MissingMethod,
	MissingRequestTarget,
	MissingHttpVersion,
	WrongHttpVersion,
	BadHeader(String),
	ReadingDoneParser,
	UnknownParserState,
}

impl fmt::Display for HttpParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			HttpParseError::MissingMethod => write!(f, "method not passed in request"),
			HttpParseError::MissingRequestTarget => write!(f, "request target not passed in request"),
			HttpParseError::MissingHttpVersion => write!(f, "missing HTTP version in request"),
			HttpParseError::WrongHttpVersion => write!(f, "unsupported HTTP version passed in request"),
			HttpParseError::BadHeader(s) => write!(f, "malformed header in request: {}", s),
			HttpParseError::ReadingDoneParser => write!(f, "reading when parser is in done state"),
			HttpParseError::UnknownParserState => write!(f, "unknown parser state"),
		}
	}
}

impl Error for HttpParseError {}
