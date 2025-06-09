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

        assert_eq!(result.get("host").unwrap(), "localhost:42069");
        assert_eq!(result.get("user-agent").unwrap(), "curl/7.81.0");
        assert_eq!(result.get("accept").unwrap(), "*/*");
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
}

