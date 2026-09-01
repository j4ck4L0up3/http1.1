use crate::http_error::HttpParseError;

#[derive(PartialEq, Debug)]
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
