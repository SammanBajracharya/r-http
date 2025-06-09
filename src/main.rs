pub mod request;
pub mod headers;

use crate::request::Request;
use std::io::BufReader;
use crate::request::test::ChunkReader;

fn main() {
    // Example usage of the Request struct
	let request =  b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 13\r\n\r\nhello world!\n";
    let chunk_reader = ChunkReader::new(request, 3);
    let mut reader = BufReader::new(chunk_reader);
    let req = Request::req_from_reader(&mut reader).expect("Failed to parse request");
    println!("Method: {:?}", req);
}
