use rand::Rng;

/// Generic RTP header builder (RFC 3550 §5.1).
///
/// Shared by all codec packetizers. Manages sequence number (u16, wrapping)
/// and timestamp (u64 internally, lower 32 bits written to header).
#[derive(Debug)]
pub struct RtpHeader {
    pub pt: u8,
    pub ssrc: u32,
    sequence: u16,
    timestamp: u64,
}

impl RtpHeader {
    pub fn new(pt: u8, ssrc: u32) -> Self {
        tracing::debug!(pt, ssrc = format_args!("{:#010X}", ssrc), "RTP header state created");
        Self {
            pt,
            ssrc,
            sequence: 0,
            timestamp: 0,
        }
    }

    /// Create with a random SSRC per RFC 3550 §8.1.
    pub fn with_random_ssrc(pt: u8) -> Self {
        let ssrc = rand::rng().random::<u32>();
        Self::new(pt, ssrc)
    }

    pub fn sequence(&self) -> u16 {
        self.sequence
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Write a 12-byte RTP header and advance the sequence number.
    pub fn write(&mut self, marker: bool) -> [u8; 12] {
        let first_byte: u8 = 2 << 6; // version=2, padding=0, extension=0, CC=0
        let second_byte: u8 = ((marker as u8) << 7) | self.pt;

        let mut header = [0u8; 12];
        header[0] = first_byte;
        header[1] = second_byte;
        header[2..4].copy_from_slice(&self.sequence.to_be_bytes());
        header[4..8].copy_from_slice(&(self.timestamp as u32).to_be_bytes());
        header[8..12].copy_from_slice(&self.ssrc.to_be_bytes());

        self.sequence = self.sequence.wrapping_add(1);
        header
    }

    /// Advance the timestamp by the given increment (typically clock_rate / fps).
    pub fn advance_timestamp(&mut self, increment: u32) {
        self.timestamp = self.timestamp.wrapping_add(increment as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_header() -> RtpHeader {
        RtpHeader::new(96, 0xAABBCCDD)
    }

    #[test]
    fn version_is_2() {
        let mut h = make_header();
        let buf = h.write(false);
        assert_eq!(buf[0] >> 6, 2);
    }

    #[test]
    fn marker_bit() {
        let mut h = make_header();
        let no_marker = h.write(false);
        assert_eq!(no_marker[1] & 0x80, 0);

        let with_marker = h.write(true);
        assert_eq!(with_marker[1] & 0x80, 0x80);
    }

    #[test]
    fn payload_type() {
        let mut h = make_header();
        let buf = h.write(false);
        assert_eq!(buf[1] & 0x7f, 96);
    }

    #[test]
    fn sequence_increments() {
        let mut h = make_header();
        let b1 = h.write(false);
        let seq1 = u16::from_be_bytes([b1[2], b1[3]]);
        let b2 = h.write(false);
        let seq2 = u16::from_be_bytes([b2[2], b2[3]]);
        assert_eq!(seq2, seq1 + 1);
    }

    #[test]
    fn sequence_wraps() {
        let mut h = make_header();
        h.sequence = u16::MAX;
        let buf = h.write(false);
        let seq = u16::from_be_bytes([buf[2], buf[3]]);
        assert_eq!(seq, u16::MAX);
        assert_eq!(h.sequence(), 0);
    }

    #[test]
    fn ssrc_written() {
        let mut h = make_header();
        let buf = h.write(false);
        let ssrc = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        assert_eq!(ssrc, 0xAABBCCDD);
    }

    #[test]
    fn timestamp_advance() {
        let mut h = make_header();
        h.advance_timestamp(3000);
        assert_eq!(h.timestamp(), 3000);
        h.advance_timestamp(3000);
        assert_eq!(h.timestamp(), 6000);
    }

    #[test]
    fn random_ssrc_differs() {
        let h1 = RtpHeader::with_random_ssrc(96);
        let h2 = RtpHeader::with_random_ssrc(96);
        assert_ne!(h1.ssrc, h2.ssrc);
    }
}
