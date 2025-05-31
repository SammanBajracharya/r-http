use std::collections::HashMap;
use std::io::{BufRead, Error, Result};
use std::str;

#[derive(Debug)]
struct RequestLine {
    http_version: String,
    method: String,
    request_target: String,
}

#[derive(Debug)]
struct Request {
    request_line: RequestLine,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn new() -> Self {
        Request {
            request_line: RequestLine {
                http_version: String::new(),
                method: String::new(),
                request_target: String::new(),
            },
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    fn read_line_as_bytes(reader: &mut dyn BufRead) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        reader.read_until(b'\n', &mut buffer)?;
        Ok(buffer)
    }

    fn is_crlf(line: &[u8]) -> bool {
        line == b"\r\n"
    }

    fn is_valid_request_line(parts: &Vec<&str>) -> bool {
        if parts.len() != 3 {
            return false; // Invalid number of parts
        }
        let method = parts[0];
        let request_target = parts[1];
        let http_version = parts[2];

        match method {
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" => {},
            _ => return false, // Invalid HTTP method
        }

        if !request_target.starts_with('/') {
            return false; // Invalid request target
        }

        if !http_version.starts_with("HTTP/") {
            return false; // Invalid HTTP version
        }

        return true
    }

    fn req_from_reader(reader: &mut dyn BufRead) -> Result<Request> {
        let mut line = Self::read_line_as_bytes(reader)?;
        if !line.ends_with(b"\r\n") {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Request line must end with CRLF",
            ));
        }

        let mut line_str = str::from_utf8(&line[..line.len() - 2])
            .map_err(|_| Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in request line"))?;

        let parts: Vec<&str> = line_str.trim_end().split_whitespace().collect();
        if !Self::is_valid_request_line(&parts) {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid request line format",
            ));
        }

        let request_line = RequestLine {
            method: parts[0].to_string(),
            request_target: parts[1].to_string(),
            http_version: parts[2].to_string(),
        };

        line = Self::read_line_as_bytes(reader)?;
        if line == b"\r\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected CRLF after request line (violates RFC 9112)",
            ));
        }

        let mut headers = HashMap::new();
        // Read header line
        loop {
            if line == b"\r\n" {
                break; // End of headers
            }
            line_str = str::from_utf8(&line[..line.len() - 2])
                .map_err(|_| Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in header line"))?;
            if let Some((key, value)) = line_str.trim_end_matches("\r\n").split_once(':') {
                headers
                    .insert(
                        key.trim().to_string(),
                        value.trim().to_string(),
                    );
            } else {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid header format",
                ));
            }

            line = Self::read_line_as_bytes(reader)?;
        }

        Ok(Request {
            request_line,
            headers,
            body: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_valid_get_request() {
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\nUser-Agent: test\r\n\r\n";
        let mut reader = BufReader::new(&request[..]);
        let req = Request::req_from_reader(&mut reader).expect("Failed to parse request");

        assert_eq!(req.request_line.method, "GET");
        assert_eq!(req.request_line.request_target, "/");
        assert_eq!(req.request_line.http_version, "HTTP/1.1");
        assert_eq!(req.headers.get("Host").unwrap(), "localhost");
        assert_eq!(req.headers.get("User-Agent").unwrap(), "test");
    }

    #[test]
    fn test_crlf_after_request_line_should_fail() {
        let request = b"GET / HTTP/1.1\r\n\r\nHost: localhost\r\n";
        let mut reader = BufReader::new(&request[..]);
        let result = Request::req_from_reader(&mut reader);
        assert!(result.is_err(), "Expected error for CRLF after request line");
    }

    #[test]
    fn test_invalid_request_line_format() {
        let request = b"/ HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut reader = BufReader::new(&request[..]);
        let result = Request::req_from_reader(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_post_request() {
        let request = b"POST /submit HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"key\":\"value\"}";
        let mut reader = BufReader::new(&request[..]);
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
        let mut reader = BufReader::new(&request[..]);
        let result = Request::req_from_reader(&mut reader);
        assert!(result.is_err(), "Expected error for invalid path in request line");
    }

    #[test]
    fn test_invalid_method_in_request_line() {
        let request = b"INVALID / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut reader = BufReader::new(&request[..]);
        let result = Request::req_from_reader(&mut reader);
        assert!(result.is_err(), "Expected error for invalid method in request line");
    }
}
