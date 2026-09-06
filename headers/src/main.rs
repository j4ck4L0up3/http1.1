use headers::Headers;

fn main() {
	let mut headers = Headers::new();
	let data: &[u8] = b"Host: localhost:42069\r\n\r\n";

	let (parsed, done) = match headers.parse_headers(data) {
		Ok(tup) => tup,
		Err(err) => panic!("expected parsed bytes, received error: {err}"),
	};

	print!("parsed: {parsed}, done: {done}");
}
