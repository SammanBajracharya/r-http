pub mod test;

use std::collections::HashMap;
use std::io::{BufRead, Error, ErrorKind, Result, Read};
use std::str;

const SINGLETON_HEADERS: &[&str] = &[
    "content-length",
    "host",
    "authorization",
    "content-type",
    "content-encoding",
    "content-range",
    "date",
];

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

    pub fn method(&self) -> &str {
        &self.request_line.method
    }

    pub fn request_target(&self) -> &str {
        &self.request_line.request_target
    }

    pub fn http_version(&self) -> &str {
        &self.request_line.http_version
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(&key.to_ascii_lowercase()).map(|s| s.as_str())
    }

    pub fn headers(&self) -> Option<HashMap<String, String>> {
        Some(self.headers.clone())
    }

    pub fn body(&self) -> Option<&[u8]> {
        if self.body.is_empty() { None }
        else { Some(&self.body) }
    }

    fn read_as_bytes(reader: &mut dyn BufRead) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut temp = Vec::new();
        loop {
            let n = reader.read_until(b'\n', &mut temp)?;
            if n == 0 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Unexpected end of stream while reading line",
                ));
            }

            buf.extend_from_slice(&temp);

            if buf.len() >= 2 && &buf[buf.len() - 2..] == b"\r\n" {
                break;
            }
        }
        Ok(buf)
    }

    fn is_crlf(line: &[u8]) -> bool {
        line == b"\r\n"
    }

    fn verify_target_url(method: &str, target: &str) -> std::io::Result<&'static str> {
        if target.starts_with("http://") || target.starts_with("https://") {
            Ok("absolute")
        } else if target.starts_with('/') {
            Ok("origin")
        } else if target == "*" {
            Ok("asterisk")
        } else if method == "CONNECT" && target.contains(":") {
            Ok("authority")
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid request target format",
            ))
        }
    }

    fn parse_request_line(reader: &mut dyn BufRead) -> std::io::Result<RequestLine> {
        let line = Self::read_as_bytes(reader)?;
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

        let request_line = RequestLine {
            method: parts[0].to_string(),
            request_target: parts[1].to_string(),
            http_version: parts[2].to_string(),
        };

        match request_line.method.as_str() {
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" | "CONNECT" => {},
            _ => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid HTTP method",
                ));
            }
        }

        if !request_line.http_version.starts_with("HTTP/") {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "HTTP version must start with 'HTTP/'",
            ));
        }

        Ok(request_line)
    }

    fn is_token_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || matches!(c,
            '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' |
            '-' | '.' | '^' | '_' | '`' | '|' | '~'
        )
    }

    pub fn parse_header_line(reader: &mut dyn BufRead) -> std::io::Result<HashMap<String, String>> {
        let mut line = Self::read_as_bytes(reader)?;
        if line == b"\r\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected CRLF after request line (violates RFC 9112)",
            ));
        }

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut line_str;

        loop {
            if line == b"\r\n" { break; }

            line_str = str::from_utf8(&line[..line.len() - 2])
                .map_err(|_| Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in header line"))?;
            if let Some((key, value)) = line_str.split_once(':') {

                if key.chars().any(|c| c.is_ascii_whitespace()) {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid header should not allow whitespace in field name",
                    ));
                } else if key.chars().any(|c| !Self::is_token_char(c)) {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid header should not allow alphanumeric characters in field name",
                    ));
                }

                let key_lower = key.to_ascii_lowercase();
                let value_trimmed = value.trim();

                if headers.contains_key(&key_lower) && SINGLETON_HEADERS.contains(&key_lower.as_str()) {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Duplicate singleton header should cause error",
                    ));
                } else if headers.contains_key(&key_lower) && !SINGLETON_HEADERS.contains(&key_lower.as_str()) {
                    println!("Warning: Duplicate header found: {}", key_lower);
                    if let Some(existing_value) = headers.get_mut(&key_lower) {
                        existing_value.push_str(", ");
                        existing_value.push_str(value_trimmed);
                    }
                } else {
                    headers.
                        insert(
                            key_lower,
                            value_trimmed.to_string(),
                        );
                }
            } else {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid header format",
                ));
            }

            line = Self::read_as_bytes(reader)?;
        }

        Ok(headers)
    }

    fn read_chunked_body(reader: &mut dyn BufRead) -> std::io::Result<Vec<u8>> {
        let mut body: Vec<u8> = Vec::new();

        loop {
            let mut size_line = String::new();
            reader.read_line(&mut size_line)?;
            let size_str = size_line.trim_end();

            let chunk_size = usize::from_str_radix(size_str, 16).map_err(|_| {
                Error::new(ErrorKind::InvalidData, "Invalid chunk size")
            })?;

            if chunk_size == 0 { break; }

            let mut chunk = vec![0; chunk_size];
            reader.read_exact(&mut chunk)?;
            body.extend(chunk);

            let mut crlf = [0u8; 2];
            reader.read_exact(&mut crlf)?;
            if &crlf != b"\r\n" {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Expected CRLF after chunk data",
                ));
            }
        }

        Ok(body)
    }

    pub fn req_from_reader(reader: &mut dyn BufRead) -> Result<Request> {
        let mut request_line = Self::parse_request_line(reader)?;

        let headers = Self::parse_header_line(reader)?;
        if headers.contains_key("content-length") && headers.contains_key("transfer-encoding") {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Cannot have both Content-Length and Transfer-Encoding headers",
            ));
        }

        let form = Self::verify_target_url(&request_line.method, &request_line.request_target)?;
        let scheme = "http";

        match form {
            "origin" => {
                let host = headers.get("host")
                    .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Host header is required for origin target"))?;
                request_line.request_target = format!("{}://{}{}", scheme, host, request_line.request_target);
            }
            "authority" => {
                request_line.request_target = format!("{}://{}", scheme, request_line.request_target);
            }
            "asterisk" | "absolute" => {}
            _ => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid request target format",
                ));
            }
        }

        let mut body: Vec<u8> = Vec::new();
        if let Some(content_length) = headers.get("content-length") {
            let len: usize = content_length.parse().map_err(|_| {
                Error::new(ErrorKind::InvalidData, "Invalid Content-Length value")
            })?;
            let mut limited = reader.take(len as u64);
            limited.read_to_end(&mut body).map_err(|e| {
                Error::new(ErrorKind::UnexpectedEof, format!("Failed to read body: {}", e))
            })?;
        } else if let Some(transfer_encoding) = headers.get("transfer-encoding") {
            if transfer_encoding != "chunked" {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Unsupported Transfer-Encoding, only 'chunked' is supported",
                ));
            }
            body = Self::read_chunked_body(reader)?;
        }

        Ok(Request {
            request_line,
            headers,
            body,
        })
    }
}
