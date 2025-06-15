use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::io::{BufReader, Write};
use std::error::Error;
use std::result::Result;

use crate::request::Request;
use crate::response::{
    HandlerError,
    StatusCode,
    Writer,
};

pub struct Server {
    listener: TcpListener,
    is_closed: Arc<AtomicBool>,
}

pub type Handler = fn(req: Request, res: &mut Writer<TcpStream>) -> Result<(), HandlerError>;

impl Drop for Server {
    fn drop(&mut self) {
        self.is_closed.store(true, Ordering::SeqCst);
    }
}

impl Server {
    pub fn start(port: u16, handler: Handler) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        let is_closed = Arc::new(AtomicBool::new(false));
        let server = Server {
            listener: listener.try_clone()?,
            is_closed: is_closed.clone(),
        };

        thread::spawn(move || {
            Self::listen(listener, is_closed, handler);
        });

        Ok(server)
    }

    fn listen(listener: TcpListener, is_closed: Arc<AtomicBool>, handler: Handler) {
        listener.set_nonblocking(true).expect("Failled to set non-blocking");

        while !is_closed.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((conn, _addr)) => {
                    thread::spawn(move || {
                        Server::handle(conn, handler);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    eprint!("Listener error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle(mut conn: TcpStream, handler: Handler) {
        let mut reader = BufReader::new(&mut conn);
        let req = match Request::req_from_reader(&mut reader) {
            Ok(r) => r,
            Err(e) => {
                let mut writer = Writer::new(&mut conn);
                let err = HandlerError {
                    status: StatusCode::BadRequest,
                    message: format!("Failed to parse request: {}\n", e),
                };

                Self::write_handler_error(&mut writer, err);
                return;
            }
        };

        drop(reader);

        let mut writer = Writer::new(&mut conn);
        if let Err(e) = handler(req, &mut writer) {
            Self::write_handler_error(&mut writer, e);
        }
    }

    fn write_handler_error(writer: &mut Writer<TcpStream>, err: HandlerError) {
        writer.set_status(err.status);
        writer.set_header("Content-Type", "text/html");
        let body = err.message.as_bytes();
        let _ = writer.write_body(body);
    }
}
