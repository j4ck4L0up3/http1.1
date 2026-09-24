use request::Request;
use std::{
	io::{BufReader, Error}, net::{IpAddr, Ipv4Addr, TcpListener, TcpStream}, sync::atomic::AtomicBool, thread, time::Duration,
};
use tokio::{sync::mpsc::{Receiver, error::TryRecvError}, self, task::JoinHandle};

pub struct Server {
	pub listening: AtomicBool,
	pub ip_addr: IpAddr,
	pub port: u16,
	handle: JoinHandle<()>,
}

impl Server {
	pub fn serve(port: u16, sig: Receiver<AtomicBool>) -> Result<Server, Error> {
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

		let handle = tokio::spawn(Self::listen(listener, sig));
		let server = Server {
			listening: AtomicBool::new(true),
			ip_addr: ip,
			port: port,
			handle,
		};

		Ok(server)
	}

	async fn listen(listener: TcpListener, mut sig: Receiver<AtomicBool>) {
		loop {
			let listening: AtomicBool = sig.try_recv().unwrap_or_else(|err| {
				match err {
					TryRecvError::Empty => AtomicBool::new(true),
					_ => {
						eprintln!("unable to receive graceful shutdown signal, initiating graceful shutdown anyway: {err}");
						AtomicBool::new(false)
					}
				}
			}) ;

			if !listening.into_inner() {
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

	fn handle(stream: TcpStream) {
		let reader = BufReader::new(stream);
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
	}
}

impl Drop for Server {
	fn drop(&mut self) {
		while !self.handle.is_finished() {
			thread::sleep(Duration::from_millis(10));
		}
	}
}
