pub mod transport;

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::Result;
pub use transport::Transport;

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

const SERVER_PORT_MIN: u64 = 5000;
const SERVER_PORT_MAX: u64 = 65534;

pub const DEFAULT_SESSION_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Ready,
    Playing,
    Paused,
}

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub uri: String,
    pub transport: RwLock<Option<Transport>>,
    pub state: RwLock<SessionState>,
    pub timeout_secs: u64,
}

impl Session {
    pub fn new(uri: &str) -> Self {
        let id = SESSION_COUNTER.fetch_add(1, Ordering::SeqCst);
        Session {
            id: format!("{:016X}", id),
            uri: uri.to_string(),
            transport: RwLock::new(None),
            state: RwLock::new(SessionState::Ready),
            timeout_secs: DEFAULT_SESSION_TIMEOUT_SECS,
        }
    }

    pub fn set_transport(&self, transport: Transport) {
        tracing::debug!(session_id = %self.id, client_addr = %transport.client_addr, "transport configured");
        *self.transport.write() = Some(transport);
    }

    pub fn get_transport(&self) -> Option<Transport> {
        self.transport.read().clone()
    }

    pub fn set_state(&self, state: SessionState) {
        tracing::debug!(session_id = %self.id, old_state = ?*self.state.read(), new_state = ?state, "state transition");
        *self.state.write() = state;
    }

    pub fn get_state(&self) -> SessionState {
        self.state.read().clone()
    }

    pub fn is_playing(&self) -> bool {
        *self.state.read() == SessionState::Playing
    }

    /// Returns the Session header value including timeout, e.g. "0000000000000001;timeout=60"
    pub fn session_header_value(&self) -> String {
        format!("{};timeout={}", self.id, self.timeout_secs)
    }
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<Session>>>>,
    next_server_port: Arc<AtomicU64>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            next_server_port: Arc::new(AtomicU64::new(SERVER_PORT_MIN)),
        }
    }

    pub fn create_session(&self, uri: &str) -> Arc<Session> {
        let session = Arc::new(Session::new(uri));
        let id = session.id.clone();
        self.sessions.write().insert(id.clone(), session.clone());

        let total = self.sessions.read().len();
        tracing::debug!(session_id = %id, uri, total_sessions = total, "session created");

        session
    }

    pub fn get_session(&self, id: &str) -> Option<Arc<Session>> {
        self.sessions.read().get(id).cloned()
    }

    pub fn remove_session(&self, id: &str) -> Option<Arc<Session>> {
        let removed = self.sessions.write().remove(id);
        if removed.is_some() {
            let total = self.sessions.read().len();
            tracing::debug!(session_id = %id, total_sessions = total, "session removed");
        }
        removed
    }

    /// Remove all sessions whose IDs are in the given list.
    pub fn remove_sessions(&self, ids: &[String]) -> usize {
        let mut sessions = self.sessions.write();
        let mut removed = 0;
        for id in ids {
            if sessions.remove(id).is_some() {
                removed += 1;
            }
        }
        if removed > 0 {
            tracing::debug!(removed, remaining = sessions.len(), "batch session cleanup");
        }
        removed
    }

    /// Allocate a pair of (RTP, RTCP) server ports.
    /// Ports wrap around when the range is exhausted.
    pub fn allocate_server_ports(&self) -> Result<(u16, u16)> {
        let rtp = self.next_server_port.fetch_add(2, Ordering::SeqCst);

        if rtp > SERVER_PORT_MAX {
            tracing::warn!(rtp, "port range exhausted, wrapping to {SERVER_PORT_MIN}");
            self.next_server_port.store(SERVER_PORT_MIN, Ordering::SeqCst);
            let rtp = self.next_server_port.fetch_add(2, Ordering::SeqCst);
            return Ok((rtp as u16, rtp as u16 + 1));
        }

        tracing::trace!(rtp_port = rtp, rtcp_port = rtp + 1, "allocated server ports");
        Ok((rtp as u16, rtp as u16 + 1))
    }

    pub fn get_playing_sessions(&self) -> Vec<Arc<Session>> {
        self.sessions
            .read()
            .values()
            .filter(|s| s.is_playing())
            .cloned()
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
