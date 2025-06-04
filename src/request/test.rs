use std::io::{self, Read};

pub struct ChunkReader {
    data: Vec<u8>,
    num_bytes_per_read: usize,
    pos: usize,
}

impl ChunkReader {
    pub fn new(data: &[u8], num_bytes_per_read: usize) -> Self {
        ChunkReader {
            data: data.to_vec(),
            num_bytes_per_read,
            pos: 0,
        }
    }
}

impl Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.data.len() {
            return Ok(0);
        }

        let remaining = self.data.len() - self.pos;
        let to_read = self.num_bytes_per_read.min(buf.len()).min(remaining);
        let end = self.pos + to_read;
        buf[..to_read].copy_from_slice(&self.data[self.pos..end]);
        self.pos = end;

        Ok(to_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::Request;
    use std::io::BufReader;

    #[test]
    fn test_valid_get_request() {
        let request = b"GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let req = Request::req_from_reader(&mut reader).expect("Failed to parse request");

        assert_eq!(req.request_line.method, "GET");
        assert_eq!(req.request_line.request_target, "/");
        assert_eq!(req.request_line.http_version, "HTTP/1.1");
        assert_eq!(req.headers.get("Host").unwrap(), "localhost:42069");
        assert_eq!(req.headers.get("User-Agent").unwrap(), "curl/7.81.0");
        assert_eq!(req.headers.get("Accept").unwrap(), "*/*");
    }

    #[test]
    fn test_crlf_after_request_line_should_fail() {
        let request = b"GET / HTTP/1.1\r\n\r\nHost: localhost\r\n";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let result = Request::req_from_reader(&mut reader);
        assert!(
            result.is_err(),
            "Expected error for CRLF after request line"
        );
    }

    #[test]
    fn test_invalid_request_line_format() {
        let request = b"/ HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let result = Request::req_from_reader(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_post_request() {
        let request = b"POST /submit HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"key\":\"value\"}";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let req = Request::req_from_reader(&mut reader).expect("Failed to parse request");

        assert_eq!(req.request_line.method, "POST");
        assert_eq!(req.request_line.request_target, "/submit");
        assert_eq!(req.request_line.http_version, "HTTP/1.1");
        assert_eq!(req.headers.get("Host").unwrap(), "localhost");
        assert_eq!(req.headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(req.headers.get("Content-Length").unwrap(), "18");
    }

    #[test]
    fn test_invalid_path_in_request_line() {
        let request = b"GET HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let result = Request::req_from_reader(&mut reader);
        assert!(
            result.is_err(),
            "Expected error for invalid path in request line"
        );
    }

    #[test]
    fn test_invalid_method_in_request_line() {
        let request = b"INVALID / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let result = Request::req_from_reader(&mut reader);
        assert!(
            result.is_err(),
            "Expected error for invalid method in request line"
        );
    }
}
