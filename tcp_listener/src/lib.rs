use std::{
	io::{prelude::*, BufReader},
	net::{TcpListener, TcpStream},
};
use tokio::{
	runtime::Runtime,
	sync::mpsc::{self, Receiver, Sender},
};

pub fn serve(rt: Runtime) {
	let listener = match TcpListener::bind("127.0.0.1:7878") {
		Ok(l) => l,
		Err(err) => panic!("Could not start TCP server {err:?}"),
	};

	for stream in listener.incoming().take(1) {
		let stream = match stream {
			Ok(s) => s,
			Err(err) => panic!("Could not connect to TCP stream {err:?}"),
		};

		println!("Connection received!");
		rt.block_on(
			async {
				let (tx, mut rx): (Sender<String>, Receiver<String>) = mpsc::channel(8);

				let tx_handle = rt.spawn(get_lines_channel(stream, tx));

				let rx_handle = rt.spawn(
					async move {
						while let Some(msg) = rx.recv().await {
							print!("{msg}");
						}
					}
				);

				match tx_handle.await {
					Ok(_) => (),
					Err(err) => panic!("Couldn't await sender: {err}"),
				};

				match rx_handle.await {
					Ok(_) => (),
					Err(err) => panic!("Couldn't await receiver: {err}"),
				};
			}
		);
	}

	println!("Connection closed!");
}

async fn get_lines_channel(mut stream: TcpStream, tx: Sender<String>) {
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

		match tx.send(part).await {
			Ok(_) => (),
			Err(err) => panic!("Could not send part of message: {err}"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
