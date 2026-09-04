use crate::http_error::HttpParseError;
use std::fmt;

#[derive(PartialEq, Clone, Debug)]
pub enum Method {
	GET,
	HEAD,
	POST,
	PUT,
	DELETE,
	CONNECT,
	OPTIONS,
	TRACE,
}

impl Method {
	pub fn parse(input: &str) -> Result<Method, HttpParseError> {
		match input {
			"GET" => Ok(Method::GET),
			"HEAD" => Ok(Method::HEAD),
			"POST" => Ok(Method::POST),
			"PUT" => Ok(Method::PUT),
			"DELETE" => Ok(Method::DELETE),
			"CONNECT" => Ok(Method::CONNECT),
			"OPTIONS" => Ok(Method::OPTIONS),
			"TRACE" => Ok(Method::TRACE),
			_ => Err(HttpParseError::MissingMethod),
		}
	}
}

impl fmt::Display for Method {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Method::GET => write!(f, "GET"),
			Method::HEAD => write!(f, "HEAD"),
			Method::POST => write!(f, "POST"),
			Method::PUT => write!(f, "PUT"),
			Method::DELETE => write!(f, "DELETE"),
			Method::CONNECT => write!(f, "CONNECT"),
			Method::OPTIONS => write!(f, "OPTIONS"),
			Method::TRACE => write!(f, "TRACE"),
		}
	}
}
