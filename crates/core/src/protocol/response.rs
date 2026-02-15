#[must_use]
pub struct RtspResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl RtspResponse {
    #[must_use]
    pub fn new(status_code: u16, status_text: &str) -> Self {
        RtspResponse {
            status_code,
            status_text: status_text.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    #[must_use]
    pub fn ok() -> Self {
        Self::new(200, "OK")
    }

    #[must_use]
    pub fn not_found() -> Self {
        Self::new(404, "Not Found")
    }

    #[must_use]
    pub fn bad_request() -> Self {
        Self::new(400, "Bad Request")
    }

    #[must_use]
    pub fn add_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn serialize(&self) -> String {
        let mut response = format!("RTSP/1.0 {} {}\r\n", self.status_code, self.status_text);

        for (name, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", name, value));
        }

        if let Some(body) = &self.body {
            response.push_str(&format!("Content-Length: {}\r\n", body.len()));
            response.push_str("\r\n");
            response.push_str(body);
        } else {
            response.push_str("\r\n");
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_no_body() {
        let resp = RtspResponse::ok()
            .add_header("CSeq", "1")
            .add_header("Public", "OPTIONS");
        let s = resp.serialize();
        assert!(s.starts_with("RTSP/1.0 200 OK\r\n"));
        assert!(s.contains("CSeq: 1\r\n"));
        assert!(s.contains("Public: OPTIONS\r\n"));
        assert!(s.ends_with("\r\n"));
    }

    #[test]
    fn serialize_with_body() {
        let resp = RtspResponse::ok()
            .add_header("CSeq", "2")
            .with_body("v=0\r\n".to_string());
        let s = resp.serialize();
        assert!(s.contains("Content-Length: 5\r\n"));
        assert!(s.ends_with("v=0\r\n"));
    }

    #[test]
    fn not_found_response() {
        let resp = RtspResponse::not_found().add_header("CSeq", "5");
        assert_eq!(resp.status_code, 404);
        let s = resp.serialize();
        assert!(s.starts_with("RTSP/1.0 404 Not Found\r\n"));
    }
}
