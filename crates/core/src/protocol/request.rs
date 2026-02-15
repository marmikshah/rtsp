use crate::error::{ParseErrorKind, RtspError};

#[derive(Debug)]
pub struct RtspRequest {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
}

impl RtspRequest {
    pub fn parse(raw: &str) -> crate::error::Result<Self> {
        let mut lines = raw.lines();

        let request_line = lines
            .next()
            .ok_or(RtspError::Parse { kind: ParseErrorKind::EmptyRequest })?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() != 3 {
            return Err(RtspError::Parse { kind: ParseErrorKind::InvalidRequestLine });
        }

        let method = parts[0].to_string();
        let uri = parts[1].to_string();
        let version = parts[2].to_string();

        if version != "RTSP/1.0" {
            tracing::warn!(version, "client sent non-RTSP/1.0 version");
        }

        let mut headers = Vec::new();

        for line in lines {
            if line.is_empty() {
                break;
            }

            let colon_pos = line
                .find(':')
                .ok_or(RtspError::Parse { kind: ParseErrorKind::InvalidHeader })?;

            let name = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();

            headers.push((name, value));
        }

        Ok(RtspRequest {
            method,
            uri,
            version,
            headers,
        })
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// CSeq (Command Sequence) numbers and orders RTSP requests & responses (RFC 2326 §12.17).
    pub fn cseq(&self) -> Option<&str> {
        self.get_header("CSeq")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_options_request() {
        let raw = "OPTIONS rtsp://localhost:8554/test RTSP/1.0\r\nCSeq: 1\r\n\r\n";
        let req = RtspRequest::parse(raw).unwrap();
        assert_eq!(req.method, "OPTIONS");
        assert_eq!(req.uri, "rtsp://localhost:8554/test");
        assert_eq!(req.version, "RTSP/1.0");
        assert_eq!(req.cseq(), Some("1"));
    }

    #[test]
    fn parse_setup_with_transport() {
        let raw = "SETUP rtsp://localhost:8554/test/track1 RTSP/1.0\r\n\
                   CSeq: 3\r\n\
                   Transport: RTP/AVP;unicast;client_port=8000-8001\r\n\r\n";
        let req = RtspRequest::parse(raw).unwrap();
        assert_eq!(req.method, "SETUP");
        assert_eq!(req.cseq(), Some("3"));
        assert_eq!(
            req.get_header("Transport"),
            Some("RTP/AVP;unicast;client_port=8000-8001")
        );
    }

    #[test]
    fn parse_empty_request() {
        assert!(RtspRequest::parse("").is_err());
    }

    #[test]
    fn parse_invalid_request_line() {
        assert!(RtspRequest::parse("JUST_A_METHOD\r\n\r\n").is_err());
    }

    #[test]
    fn header_lookup_case_insensitive() {
        let raw = "OPTIONS rtsp://localhost RTSP/1.0\r\ncseq: 42\r\n\r\n";
        let req = RtspRequest::parse(raw).unwrap();
        assert_eq!(req.get_header("CSeq"), Some("42"));
        assert_eq!(req.get_header("cseq"), Some("42"));
        assert_eq!(req.get_header("CSEQ"), Some("42"));
    }
}
