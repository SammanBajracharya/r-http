use std::io::Read;

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
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
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
    use std::io::BufReader;

    use crate::headers::Headers;

    use super::*;

    #[test]
    fn valid_headers() {
        let request = b"Host: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let result = Headers::parse_header_line(&mut reader).expect("Failed to parse request");

        assert_eq!(result.get_value("host").unwrap(), "localhost:42069");
        assert_eq!(result.get_value("user-agent").unwrap(), "curl/7.81.0");
        assert_eq!(result.get_value("accept").unwrap(), "*/*");
    }

    #[test]
    fn invalid_space_bet_field_and_colon() {
        let request = b"Host : localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 3);
        let mut reader = BufReader::new(chunk_reader);
        let result = Headers::parse_header_line(&mut reader);

        assert!(
            result.is_err(),
            "Invalid header should not allow space before colon"
        );
    }

    #[test]
    fn test_duplicate_singleton_header() {
        let request = b"Host: example.com\r\nHost: duplicate.com\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 5);
        let mut reader = std::io::BufReader::new(chunk_reader);
        let result = Headers::parse_header_line(&mut reader);
        assert!(
            result.is_err(),
            "Duplicate singleton header should cause error"
        );
    }


    #[test]
    fn test_multiple_same_non_singleton_header() {
        let request = b"Accept: text/html\r\nAccept: application/json\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 4);
        let mut reader = std::io::BufReader::new(chunk_reader);
        let result = Headers::parse_header_line(&mut reader).expect("Failed to parse headers");
        assert_eq!(
            result.get_value("accept").unwrap(),
            "text/html, application/json",
        );
    }

    #[test]
    fn test_header_with_invalid_characters() {
        let request = b"Bad@Header: value\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 5);
        let mut reader = std::io::BufReader::new(chunk_reader);
        let result = Headers::parse_header_line(&mut reader);
        assert!(
            result.is_err(),
            "Invalid header should not allow alphanumeric characters in field name",
        );
    }

    #[test]
    fn test_header_line_without_colon() {
        let request = b"InvalidHeaderLine\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 5);
        let mut reader = std::io::BufReader::new(chunk_reader);
        let result = Headers::parse_header_line(&mut reader);
        assert!(
            result.is_err(),
            "Invalid header format",
        );
    }

    #[test]
    fn test_empty_header_value() {
        let request = b"X-Empty-Header:\r\n\r\n";
        let chunk_reader = ChunkReader::new(request, 5);
        let mut reader = std::io::BufReader::new(chunk_reader);
        let result = Headers::parse_header_line(&mut reader).expect("Failed to parse headers");
        assert_eq!(
            result.get_value("x-empty-header").unwrap(),
            "",
            "Empty header value should be allowed and parsed as empty string"
        );
    }
}

