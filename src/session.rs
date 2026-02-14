use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct Transport {
    pub client_rtp_port: u16,
    pub client_rtcp_port: u16,
    pub server_rtp_port: u16,
    pub server_rtcp_port: u16,
}

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub uri: String,
    pub transport: Option<Transport>,
    pub is_playing: bool,
}

impl Session {
    pub fn new(uri: &str) -> Self {
        let id = SESSION_COUNTER.fetch_add(1, Ordering::SeqCst);
        Session {
            id: format!("{:016X}", id),
            uri: uri.to_string(),
            transport: None,
            is_playing: false,
        }
    }
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: HashMap::new(),
        }
    }
    pub fn create_session(&mut self, uri: &str) -> &Session {
        let session = Session::new(uri);
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.sessions.get(&id).unwrap()
    }

    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    pub fn remove_session(&mut self, id: &str) -> Option<Session> {
        self.sessions.remove(id)
    }
}
