use std::{
	io::{self, prelude::*},
	net::UdpSocket,
};

pub fn serve() {
	let socket = match UdpSocket::bind("0.0.0.0:0") {
		Ok(s) => s,
		Err(err) => panic!("Could not bind UDP socket: {err}"),
	};

	match socket.connect("127.0.0.1:7878") {
		Ok(_) => (),
		Err(err) => panic!("Socket could not connect: {err}"),
	};

	let stdin = io::stdin();
	let mut reader = stdin.lock();

	loop {
		print!("> ");
		// must flush stdout to end with space not newline
		match io::stdout().flush() {
			Ok(_) => (),
			Err(err) => panic!("Could not flush stdout buffer: {err}"),
		}

		let mut line = String::new();

		match reader.read_line(&mut line) {
			Ok(_) => (),
			Err(err) => panic!("Unable to read line from stdin: {err}"),
		};

		match socket.send(&line.as_bytes()) {
			Ok(_) => (),
			Err(err) => panic!("Unable to send line from socket: {err}"),
		};
	}
}
