use std::io::{self, ErrorKind, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use r_http::request::Request;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:42069")?;
    println!("Connected to server at port 42069");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client conned from {:}", stream.peer_addr()?);
                thread::spawn(move || handle_client(&stream));
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(stream: &TcpStream) {
    let peer = stream.peer_addr().ok();
    let mut reader = BufReader::new(stream);

    match Request::req_from_reader(&mut reader) {
        Ok(req) => {
            println!("--- Request from {:?} ---", peer);
            println!("Method: {}", req.method());
            println!("Target: {}", req.request_target());
            println!("Version: {}", req.http_version());
            println!("--- Headers ---");
            match req.headers() {
                Some(headers) => {
                    for (k, v) in headers.iter() {
                        println!("{} => {}", k, v);
                    }
                }
                None => {}
            }
        }
        Err(e) => {
            eprintln!("Error parsing request from {:?}: {}", peer, e);
        }
    }
}

// fn get_lines_channel(mut stream: TcpStream) -> mpsc::Receiver<String> {
//     let (sx, rx) = mpsc::channel();
//     let mut buffer = [0u8; 8];
//     let mut current_line_contents = String::new();
//
//     thread::spawn(move || {
//         loop {
//             let n = match stream.read(&mut buffer) {
//                 Ok(0) => break,
//                 Ok(n) => n,
//                 Err(e) => {
//                     if e.kind() == ErrorKind::UnexpectedEof {
//                         break;
//                     } else {
//                         eprintln!("error: {}", e);
//                         break;
//                     }
//                 }
//             };
//
//             let chunk = match std::str::from_utf8(&buffer[..n]) {
//                 Ok(v) => v,
//                 Err(e) => {
//                     eprintln!("error: {}", e);
//                     continue;
//                 }
//             };
//
//             let parts: Vec<&str> = chunk.split('\n').collect();
//             for part in parts.iter().take(parts.len() - 1) {
//                 let full_line = format!("{}{}", current_line_contents, part);
//                 if sx.send(full_line).is_err() {
//                     return;
//                 }
//                 current_line_contents.clear();
//             }
//
//             current_line_contents.push_str(parts.last().unwrap_or(&""));
//         }
//
//         if !current_line_contents.is_empty() {
//             let _ = sx.send(current_line_contents);
//         }
//     });
//
//     rx
// }
