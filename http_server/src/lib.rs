use request::Request;
use std::{
	io::{BufReader, Error, Write}, net::{IpAddr, Ipv4Addr, TcpListener, TcpStream, Shutdown}, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, time::Duration,
};
use tokio::{sync::broadcast::{Receiver, error::TryRecvError}, self, task::JoinHandle};

#[derive(Debug)]
pub struct Server {
	pub listening: Arc<AtomicBool>,
	pub ip_addr: IpAddr,
	pub port: u16,
	handle: JoinHandle<()>,
}

impl Server {
	pub fn serve(port: u16, sig: Receiver<bool>) -> Result<Server, Error> {
		let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
		let parsed_port = port.to_string();
		let address = ip.to_string() + ":" + parsed_port.as_str();
		let listener = TcpListener::bind(&address).map_err(|err| {
			Error::new(
				err.kind(),
				format!(
					"unable to create listener from address {:?}: {err}",
					address,
				),
			)
		})?;

		listener.set_nonblocking(true)?;
		let listening = Arc::new(AtomicBool::new(true));
		let listening_clone = Arc::clone(&listening);

		let handle = tokio::spawn(Self::listen(listener, sig, listening_clone));
		let server = Server {
			listening,
			ip_addr: ip,
			port: port,
			handle,
		};

		Ok(server)
	}

	async fn listen(listener: TcpListener, mut sig: Receiver<bool>, flag: Arc<AtomicBool>) {
		loop {
			let listening: bool = sig.try_recv().unwrap_or_else(|err| {
				match err {
					TryRecvError::Empty => true,
					_ => {
						eprintln!("unable to receive graceful shutdown signal, initiating graceful shutdown anyway: {err}");
						false
					}
				}
			}) ;

			if !listening {
				flag.store(listening, Ordering::Release);
				return;
			}

			let (socket, _) = match listener.accept() {
				Ok(tup) => tup,
				Err(_) => {
					continue;
				}
			};

			Self::handle(socket);
		}
	}

	fn handle(mut stream: TcpStream) {
		let reader = BufReader::new(&stream);
		let request = match Request::from_reader(reader) {
			Ok(req) => req,
			Err(err) => panic!("Error with parsed request {err}"),
		};

		println!(
			"Request line:\n- Method: {}\n- Target: {}\n- Version: {}\nHeaders:",
			request.method.unwrap(),
			request.request_target,
			request.http_version,
		);

		for key in request.headers.field_lines.keys() {
			println!("- {}: {}", &*key, request.headers.get(key));
		}

		if !request.body.is_empty() {
			println!(
				"Body:\n{}",
				std::str::from_utf8(&*request.body.to_vec()).unwrap()
			);
		}
		
		let response = 
			b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\nHello World!";
		
		let _ = stream.write(response).map_err(|err| eprintln!("unable to write response to TCP connection: {err}"));
		let _ = stream.shutdown(Shutdown::Both).map_err(|err| eprintln!("unable to close TCP connection: {err}"));
	}
}

impl Drop for Server {
	fn drop(&mut self) {
		while !self.handle.is_finished() {
			thread::sleep(Duration::from_millis(10));
		}
	}
}
