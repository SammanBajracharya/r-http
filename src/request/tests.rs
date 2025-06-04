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
