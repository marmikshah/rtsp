use crate::protocol::{parse_transport_header, RtspRequest, RtspResponse};
use crate::session::{SessionManager, Transport};

pub struct RequestHandler {
    session_manager: SessionManager,
    next_server_port: u16,
}

impl RequestHandler {
    pub fn new() -> Self {
        RequestHandler {
            session_manager: SessionManager::new(),
            next_server_port: 6000,
        }
    }

    pub fn handle(&mut self, request: &RtspRequest) -> RtspResponse {
        let cseq = request.cseq().unwrap_or("0");

        match request.method.as_str() {
            "OPTIONS" => self.handle_options(cseq),
            "DESCRIBE" => self.handle_describe(cseq, &request.uri),
            "SETUP" => self.handle_setup(cseq, &request),
            "PLAY" => self.handle_play(cseq, &request),
            "PAUSE" => self.handle_pause(cseq, &request),
            "TEARDOWN" => self.handle_teardown(cseq, &request),
            _ => RtspResponse::new(501, "Not Implemented").add_header("CSeq", cseq),
        }
    }

    fn handle_options(&self, cseq: &str) -> RtspResponse {
        RtspResponse::ok()
            .add_header("CSeq", cseq)
            .add_header("Public", "OPTIONS, DESCRIBE, SETUP, PLAY, PAUSE, TEARDOWN")
    }

    fn handle_describe(&self, cseq: &str, uri: &str) -> RtspResponse {
        // A minimal SDP for Single H264 stream
        // Session Desciption Protocol (SDP)
        let sdp = format!(
            "v=0\r\n
            o=- 0.0 IN IP4 127.0.0.1\r\n\
            s=RTSP Server\r\n
            c=IN IP4 0.0.0.0\r\n\
            t=0 0\r\n\
            m=Video 0 RTP/AVP 96\r\n\
            a=rtpmap:96 H264/90000\r\n\
            a=control:track1\r\n"
        );

        RtspResponse::ok()
            .add_header("CSeq", cseq)
            .add_header("Content-Type", "application/sdp")
            .add_header("Content-Base", uri)
            .with_body(sdp)
    }

    fn handle_setup(&mut self, cseq: &str, request: &RtspRequest) -> RtspResponse {
        let transport_header = match request.get_header("Transport") {
            Some(t) => t,
            None => {
                return RtspResponse::bad_request().add_header("CSeq", cseq);
            }
        };

        let client_transport = match parse_transport_header(transport_header) {
            Some(t) => t,
            None => {
                return RtspResponse::bad_request().add_header("CSeq", cseq);
            }
        };

        let server_rtp_port = self.next_server_port;
        let server_rtcp_port = self.next_server_port + 1;
        self.next_server_port += 2;

        let session = self.session_manager.create_session(&request.uri);

        let session_id = session.id.clone();

        if let Some(session) = self.session_manager.get_session_mut(&session_id) {
            session.transport = Some(Transport {
                client_rtp_port: client_transport.client_rtp_port,
                client_rtcp_port: client_transport.client_rtcp_port,
                server_rtp_port,
                server_rtcp_port,
            });
        }

        let transport_response = format!(
            "RTP/AVP;unicast;client_port={}-{};server_port={}-{}",
            client_transport.client_rtp_port,
            client_transport.client_rtcp_port,
            server_rtp_port,
            server_rtcp_port
        );

        RtspResponse::ok()
            .add_header("CSeq", cseq)
            .add_header("Transport", &transport_response)
            .add_header("Session", &session_id)
    }

    fn handle_play(&mut self, cseq: &str, request: &RtspRequest) -> RtspResponse {
        let session_id = match request.get_header("Session") {
            Some(s) => s,
            None => {
                return RtspResponse::new(454, "Session Not Found").add_header("CSeq", cseq);
            }
        };

        match self.session_manager.get_session_mut(session_id) {
            Some(session) => {
                session.is_playing = true;
                println!("Session {} started playing", session_id);
                RtspResponse::ok()
                    .add_header("CSeq", cseq)
                    .add_header("Session", session_id)
                    .add_header("Range", "npt=0.000-")
            }
            None => RtspResponse::new(454, "Session Not Found").add_header("CSeq", cseq),
        }
    }

    fn handle_pause(&mut self, cseq: &str, request: &RtspRequest) -> RtspResponse {
        let session_id = match request.get_header("Session") {
            Some(s) => s,
            None => {
                return RtspResponse::new(454, "Session Not Found").add_header("CSeq", cseq);
            }
        };

        match self.session_manager.get_session_mut(session_id) {
            Some(session) => {
                session.is_playing = false;
                println!("Session {} paused", session_id);

                RtspResponse::ok()
                    .add_header("CSeq", cseq)
                    .add_header("Session", session_id)
            }
            None => RtspResponse::new(454, "Session Not Found").add_header("CSeq", cseq),
        }
    }

    fn handle_teardown(&mut self, cseq: &str, request: &RtspRequest) -> RtspResponse {
        let session_id = match request.get_header("Session") {
            Some(s) => s,
            None => {
                return RtspResponse::new(454, "Session Not Found").add_header("CSeq", cseq);
            }
        };

        match self.session_manager.remove_session(session_id) {
            Some(_) => {
                println!("Session {} terminated", session_id);

                RtspResponse::ok().add_header("CSeq", cseq)
            }
            None => RtspResponse::new(454, "Session Not Found").add_header("CSeq", cseq),
        }
    }
}
