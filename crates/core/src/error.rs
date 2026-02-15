use std::fmt;

/// Errors that can occur in the RTSP server library.
///
/// These map to specific failure modes in the RTSP/RTP stack:
/// - Protocol-level errors (parsing, invalid state)
/// - Transport-level errors (I/O, socket failures)
/// - Session-level errors (not found, invalid state)
#[derive(Debug, thiserror::Error)]
pub enum RtspError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("transport not configured for session: {0}")]
    TransportNotConfigured(String),

    #[error("session not in playing state: {0}")]
    SessionNotPlaying(String),

    #[error("server not started")]
    NotStarted,

    #[error("server already running")]
    AlreadyRunning,

    #[error("RTSP parse error: {kind}")]
    Parse { kind: ParseErrorKind },

    #[error("port range exhausted (tried to allocate beyond u16 range)")]
    PortRangeExhausted,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    EmptyRequest,
    InvalidRequestLine,
    InvalidHeader,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRequest => write!(f, "empty request"),
            Self::InvalidRequestLine => write!(f, "invalid request line"),
            Self::InvalidHeader => write!(f, "invalid header"),
        }
    }
}

pub type Result<T> = std::result::Result<T, RtspError>;
