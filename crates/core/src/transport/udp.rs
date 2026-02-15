use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;

use crate::error::{Result, RtspError};
use crate::session::SessionManager;

/// UDP transport for RTP packet delivery.
pub struct UdpTransport {
    socket: Arc<UdpSocket>,
}

impl UdpTransport {
    /// Bind an ephemeral UDP socket for outbound RTP.
    pub fn bind() -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        Ok(Self {
            socket: Arc::new(socket),
        })
    }

    /// Send an RTP packet to a specific session.
    pub fn send_to_session(
        &self,
        session_manager: &SessionManager,
        session_id: &str,
        payload: &[u8],
    ) -> Result<usize> {
        let session = session_manager
            .get_session(session_id)
            .ok_or_else(|| RtspError::SessionNotFound(session_id.to_string()))?;

        if !session.is_playing() {
            return Err(RtspError::SessionNotPlaying(session_id.to_string()));
        }

        let transport = session
            .get_transport()
            .ok_or_else(|| RtspError::TransportNotConfigured(session_id.to_string()))?;

        Ok(self.socket.send_to(payload, transport.client_addr)?)
    }

    /// Broadcast an RTP packet to all playing sessions.
    pub fn broadcast(
        &self,
        session_manager: &SessionManager,
        payload: &[u8],
    ) -> Result<usize> {
        let playing = session_manager.get_playing_sessions();

        if playing.is_empty() {
            return Ok(0);
        }

        let mut sent = 0;
        for session in &playing {
            if let Some(transport) = session.get_transport() {
                match self.socket.send_to(payload, transport.client_addr) {
                    Ok(_) => sent += 1,
                    Err(e) => {
                        tracing::warn!(
                            session_id = %session.id,
                            addr = %transport.client_addr,
                            error = %e,
                            "failed to send RTP packet"
                        );
                    }
                }
            }
        }

        Ok(sent)
    }

    /// Send raw bytes to a specific address.
    pub fn send_to(&self, payload: &[u8], addr: SocketAddr) -> Result<usize> {
        Ok(self.socket.send_to(payload, addr)?)
    }
}
