pub mod test;

use std::collections::HashMap;
use std::io::{BufRead, Error, ErrorKind, Result};
use std::str;

#[derive(Debug)]
pub struct RequestLine {
    http_version: String,
    method: String,
    request_target: String,
}

#[derive(Debug)]
pub struct Request {
    request_line: RequestLine,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    pub fn new() -> Self {
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

    fn read_full_line(reader: &mut dyn BufRead) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8, 1];
        const MAX_LINE_LENGTH: usize = 8192; // Arbitrary limit to prevent excessive memory usage

        loop {
            let n = reader.read(&mut temp_buf)?;
            if n == 0 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Unexpected end of stream while reading line",
                ));
            }

            if buffer.len() > MAX_LINE_LENGTH {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Line exceeds maximum length",
                ));
            }

            buffer.push(temp_buf[0]);
            if buffer.len() >= 2 && &buffer[buffer.len() - 2..] == b"\r\n" {
                return Ok(buffer);
            }
        }
    }

    fn is_crlf(line: &[u8]) -> bool {
        line == b"\r\n"
    }

    fn parse_request_line(reader: &mut dyn BufRead) -> std::io::Result<(String, String, String)> {
        let line = Self::read_full_line(reader)?;
        if line.len() < 2 || !line.ends_with(b"\r\n") {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Request line must end with CRLF",
            ));
        }

        let line_str = str::from_utf8(&line[..line.len() - 2])
            .map_err(|_| Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in request line"))?;

        let parts: Vec<&str> = line_str.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Request line must have exactly three parts: method, request target, and HTTP version",
            ));
        }

        let method = parts[0];
        let request_target = parts[1];
        let http_version = parts[2];

        match method {
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" => {},
            _ => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid HTTP method",
                ));
            }
        }

        if !request_target.starts_with('/') {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Request target must start with '/'",
            ));
        }

        if !http_version.starts_with("HTTP/") {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "HTTP version must start with 'HTTP/'",
            ));
        }

        Ok((method.to_string(), request_target.to_string(), http_version.to_string()))
    }

    fn parse_header_line(reader: &mut dyn BufRead) -> std::io::Result<HashMap<String, String>> {
        let mut line = Self::read_full_line(reader)?;
        if line == b"\r\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected CRLF after request line (violates RFC 9112)",
            ));
        }

        let mut headers = HashMap::new();
        let mut line_str;

        loop {
            if line == b"\r\n" { break; }

            line_str = str::from_utf8(&line[..line.len() - 2])
                .map_err(|_| Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in header line"))?;
            if let Some((key, value)) = line_str.trim_end_matches("\r\n").split_once(':') {
                let key_lower = key.trim().to_ascii_lowercase();
                // if headers.contains_key(&key_lower) {
                //     return Err(Error::new(
                //         std::io::ErrorKind::InvalidData,
                //         "Duplicate header found",
                //     ));
                // }

                headers.
                    insert(
                        key_lower,
                        value.trim().to_string(),
                    );
            } else {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid header format",
                ));
            }

            line = Self::read_full_line(reader)?;
        }

        Ok(headers)
    }

    pub fn req_from_reader(reader: &mut dyn BufRead) -> Result<Request> {
        let parts = Self::parse_request_line(reader)?;

        let request_line = RequestLine {
            method: parts.0.to_string(),
            request_target: parts.1.to_string(),
            http_version: parts.2.to_string(),
        };

        let headers = Self::parse_header_line(reader)?;

        Ok(Request {
            request_line,
            headers,
            body: Vec::new(),
        })
    }
}
