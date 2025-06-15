use std::collections::HashMap;
use std::io::Write;

pub struct HandlerError {
    pub status: StatusCode,
    pub message: String,
}

#[derive(Clone)]
pub enum StatusCode {
    Ok,
    NotFound,
    InternalServerError,
    BadRequest,
}

pub enum WriterState {
    Init,
    StatusWritten,
    HeadersWritten,
    BodyWritten,
}

pub struct Writer<'a, W: Write> {
    inner: &'a mut W,
    headers: HashMap<String, String>,
    status: Option<StatusCode>,
    state: WriterState,
}

impl<'a, W: Write> Writer<'a, W> {
    pub fn new(inner: &'a mut W) -> Self {
        Writer {
            inner,
            headers: HashMap::new(),
            status: None,
            state: WriterState::Init,
        }
    }

    pub fn set_status(&mut self, status: StatusCode) {
        if matches!(self.state, WriterState::Init) {
            self.status = Some(status);
            self.state = WriterState::StatusWritten;
        } else {
            panic!("Cannot set status after writing headers or body");
        }
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        if matches!(self.state, WriterState::StatusWritten) {
            self.headers.insert(key.to_string(), value.to_string());
        } else {
            panic!("Cannot set headers after body is written");
        }
    }

    pub fn write_body(&mut self, body: &[u8]) -> std::io::Result<()> {
        if !matches!(self.state, WriterState::BodyWritten) {
            self.flush_headers(body.len())?;
            self.inner.write_all(body)?;
            self.state = WriterState::BodyWritten;
        }
        Ok(())
    }

    fn flush_headers(&mut self, content_length: usize) -> std::io::Result<()> {
        let status_line = match self.status.as_ref().unwrap_or(&StatusCode::Ok) {
            StatusCode::Ok => "HTTP/1.1 200 OK\r\n",
            StatusCode::BadRequest => "HTTP/1.1 400 Bad Request\r\n",
            StatusCode::NotFound => "HTTP/1.1 404 Not Found\r\n",
            StatusCode::InternalServerError => "HTTP/1.1 500 Internal Server Error\r\n",
        };
        self.inner.write_all(status_line.as_bytes())?;

        self.headers
            .entry("Content-Length".to_string())
            .or_insert(content_length.to_string());
        self.headers
            .entry("Content-Type".to_string())
            .or_insert("text/plain; charset=utf-8".to_string());
        self.headers
            .entry("Connection".to_string())
            .or_insert("close".to_string());

        for (k, v) in &self.headers {
            let line = format!("{}: {}\r\n", k, v);
            self.inner.write_all(line.as_bytes())?;
        }
        self.inner.write_all(b"\r\n")?;
        self.state = WriterState::HeadersWritten;
        Ok(())
    }
}
