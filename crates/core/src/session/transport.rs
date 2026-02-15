use std::net::SocketAddr;

/// RTP/RTCP transport parameters negotiated during SETUP (RFC 2326 §12.39).
#[derive(Debug, Clone)]
pub struct Transport {
    pub client_rtp_port: u16,
    pub client_rtcp_port: u16,
    pub server_rtp_port: u16,
    pub server_rtcp_port: u16,
    pub client_addr: SocketAddr,
}

/// Parsed client-side transport info from the RTSP Transport header.
#[derive(Debug, Clone)]
pub struct TransportHeader {
    pub client_rtp_port: u16,
    pub client_rtcp_port: u16,
}

impl TransportHeader {
    /// Parse the RTSP Transport header value (RFC 2326 §12.39).
    /// Extracts client_port RTP-RTCP pair from e.g. "RTP/AVP;unicast;client_port=8000-8001".
    pub fn parse(header: &str) -> Option<Self> {
        for part in header.split(';') {
            let part = part.trim();
            if part.starts_with("client_port=") {
                let ports = &part["client_port=".len()..];
                let port_parts: Vec<&str> = ports.split('-').collect();

                if port_parts.len() == 2 {
                    let rtp_port: u16 = port_parts[0].parse().ok()?;
                    let rtcp_port: u16 = port_parts[1].parse().ok()?;

                    return Some(TransportHeader {
                        client_rtp_port: rtp_port,
                        client_rtcp_port: rtcp_port,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_transport() {
        let th = TransportHeader::parse("RTP/AVP;unicast;client_port=5000-5001").unwrap();
        assert_eq!(th.client_rtp_port, 5000);
        assert_eq!(th.client_rtcp_port, 5001);
    }

    #[test]
    fn parse_no_client_port() {
        assert!(TransportHeader::parse("RTP/AVP;unicast").is_none());
    }
}
