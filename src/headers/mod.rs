mod test;

use core::str;
use std::collections::HashMap;
use std::io::{BufRead, Error, ErrorKind};
use std::ops::Deref;

#[derive(Debug)]
pub struct Headers(HashMap<String, (String, String)>);

impl Deref for Headers {
    type Target = HashMap<String, (String, String)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const SINGLETON_HEADERS: &[&str] = &[
    "content-length",
    "host",
    "authorization",
    "content-type",
    "content-encoding",
    "content-range",
    "date",
];

impl Headers {
    fn read_as_bytes(reader: &mut dyn BufRead) -> std::io::Result<Vec<u8>> {
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

    fn is_token_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || matches!(c,
            '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' |
            '-' | '.' | '^' | '_' | '`' | '|' | '~'
        )
    }

    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.0.get(&key.to_ascii_lowercase()).map(|(_, v)| v.as_str())
    }

    pub fn original_name(&self, key: &str) -> Option<&str> {
        self.0.get(&key.to_ascii_lowercase()).map(|(k, _)| k.as_str())
    }

    pub fn parse_header_line(reader: &mut dyn BufRead) -> std::io::Result<Headers> {
        let mut line = Self::read_as_bytes(reader)?;
        if line == b"\r\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected CRLF after request line (violates RFC 9112)",
            ));
        }

        let mut headers: HashMap<String, (String, String)> = HashMap::new();
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
                    if let Some((_, existing_value)) = headers.get_mut(&key_lower) {
                        existing_value.push_str(", ");
                        existing_value.push_str(value_trimmed);
                    }
                } else {
                    headers.
                        insert(
                            key_lower,
                            (key.to_string(), value_trimmed.to_string()),
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

        Ok(Headers(headers))
    }
}
