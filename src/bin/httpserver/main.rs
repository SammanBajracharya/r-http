use std::io::{Write};
use std::result::Result;
use std::sync::{ atomic::{ AtomicBool, Ordering }, Arc };
use std::thread;
use std::time::Duration;

use r_http::response::Writer;
use r_http::{
    request::Request,
    response::{HandlerError, StatusCode},
    server::{Server},
};

const PORT: u16 = 42069;

fn handler(req: Request, res: &mut Writer<impl Write>) -> Result<(), HandlerError> {
    match req.path_segments().as_slice() {
        ["yourproblem"] => {
            res.set_status(StatusCode::BadRequest);
            res.set_header("Content-Type", "text/html");
            let _ = res.write_body(b"<html><body><h1>Bad Request</h1></body></html>");
        }
        ["myproblem"] => {
            res.set_status(StatusCode::InternalServerError);
            res.set_header("Content-Type", "text/html");
            let _ = res.write_body(b"<html><body><h1>Internal Server Error</h1></body></html>");
        }
        _ => {
            res.set_status(StatusCode::Ok);
            res.set_header("Content-Type", "text/html");
            let _ = res.write_body(b"<html><body><h1>Success!</h1></body></html>");
        }
    }
    Ok(())
}

pub fn main() {
    let server = Server::start(PORT, handler).expect("Failed to start server");
    let running = Arc::new(AtomicBool::new(true));

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }
    drop(server);
}

