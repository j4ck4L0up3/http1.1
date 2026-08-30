use std::{
	io::{prelude::*, BufReader},
	net::{TcpListener, TcpStream},
	sync::mpsc::{self, Receiver, Sender},
};

pub fn serve() {
	let listener = match TcpListener::bind("127.0.0.1:7878") {
		Ok(l) => l,
		Err(err) => panic!("Could not start TCP server {err:?}"),
	};

	for stream in listener.incoming() {
		let stream = match stream {
			Ok(s) => s,
			Err(err) => panic!("Could not connect to TCP stream {err:?}"),
		};

		println!("Connection received!");
		let rx = get_lines_channel(stream);
		let msg = rx.recv();

		match msg {
			Ok(msg) => println!("{msg}"),
			Err(err) => panic!("Could not receive msg: {err}"),
		};
	}

	println!("Connection closed!");
}

fn get_lines_channel(mut stream: TcpStream) -> Receiver<String> {
	let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

	let mut reader = BufReader::with_capacity(8, &mut stream);
	let mut curr_line = String::new();

	loop {
		let mut part = String::new();

		let _ = reader.read_line(&mut curr_line);

		if curr_line.is_empty() {
			break;
		}

		if curr_line.contains("\n") {
			let part_end = match curr_line.find("\n") {
				Some(idx) => idx,
				None => break,
			};

			let mut segment = curr_line;
			curr_line = segment.split_off(part_end + 1); // split_off is not inclusive
			part.push_str(segment.as_str());
		}

		match tx.send(part) {
			Ok(_) => (),
			Err(err) => panic!("Could not send part of message: {err}"),
		}
	}

	rx
}

#[cfg(test)]
mod tests {
	use super::*;
}
