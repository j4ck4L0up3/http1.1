use std::{error::Error, fmt};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum HttpParseError {
	MissingMethod,
	MissingRequestTarget,
	MissingHttpVersion,
	RequestLineParseError,
	WrongHttpVersion,
	BadHeader,
	ReadingDoneParser,
	UnknownParserState,
}

impl fmt::Display for HttpParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			HttpParseError::MissingMethod => write!(f, "method not passed in request"),
			HttpParseError::MissingRequestTarget => write!(f, "request target not passed in request"),
			HttpParseError::MissingHttpVersion => write!(f, "missing HTTP version in request"),
			HttpParseError::RequestLineParseError => {
				write!(f, "error while parsing request line")
			}
			HttpParseError::WrongHttpVersion => write!(f, "unsupported HTTP version passed in request"),
			HttpParseError::BadHeader => write!(f, "malformed header in request"),
			HttpParseError::ReadingDoneParser => write!(f, "reading when parser is in done state"),
			HttpParseError::UnknownParserState => write!(f, "unknown parser state"),
		}
	}
}

impl Error for HttpParseError {}
