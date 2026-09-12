use std::{error::Error, fmt};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum HttpParseError {
	MissingMethod,
	MissingRequestTarget,
	MissingHttpVersion,
	WrongHttpVersion,
	RequestLineParseError,
	HeaderParseError,
	NoColonInHeader,
	InvalidHeaderWhitespace,
	InvalidHeaderChars,
	InvalidDuplicateHeader,
	EmptyFieldValue,
	MissingEndOfHeaders,
	InvalidASCII,
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
			HttpParseError::RequestLineParseError => {
				write!(f, "error while parsing request line")
			}
			HttpParseError::HeaderParseError => write!(f, "unable to parse request headers"),
			HttpParseError::NoColonInHeader => write!(f, "colon not passed in request header"),
			HttpParseError::InvalidHeaderWhitespace => {
				write!(f, "whitespace passed in request header field name")
			}
			HttpParseError::InvalidHeaderChars => write!(f, "invalid character passed in header"),
			HttpParseError::InvalidDuplicateHeader => {
				write!(f, "invalid duplicate header passed in request")
			}
			HttpParseError::EmptyFieldValue => write!(f, "empty field value passed in header"),
			HttpParseError::MissingEndOfHeaders => write!(f, "missing CRLF at end of headers"),
			HttpParseError::InvalidASCII => write!(f, "non-ASCII bytes passed in request"),
			HttpParseError::ReadingDoneParser => write!(f, "reading when parser is in done state"),
			HttpParseError::UnknownParserState => write!(f, "unknown parser state"),
		}
	}
}

impl Error for HttpParseError {}
