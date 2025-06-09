pub mod request;
pub mod headers;

use crate::{headers::Headers, request::Request};
use std::io::BufReader;
use crate::request::test::ChunkReader;

fn main() {
    // Example usage of the Request struct
    let request = b"X-Empty-Header:\r\n\r\n";
    let chunk_reader = ChunkReader::new(request, 3);
    let mut reader = BufReader::new(chunk_reader);
    let result = Headers::parse_header_line(&mut reader).expect("Failed to parse headers");
    println!("{:?}", result);
}
