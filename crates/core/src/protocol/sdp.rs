use crate::media::Packetizer;

/// Generate an SDP session description for the given packetizer.
///
/// Produces SDP per RFC 4566 with media-level attributes from the codec's
/// [`Packetizer::sdp_attributes`] implementation.
pub fn generate_sdp(packetizer: &dyn Packetizer) -> String {
    let pt = packetizer.payload_type();
    let clock = packetizer.clock_rate();
    let codec = packetizer.codec_name();

    let mut sdp = format!(
        "v=0\r\n\
         o=- 0 0 IN IP4 127.0.0.1\r\n\
         s=RTSP Server\r\n\
         c=IN IP4 0.0.0.0\r\n\
         t=0 0\r\n\
         m=video 0 RTP/AVP {pt}\r\n\
         a=rtpmap:{pt} {codec}/{clock}\r\n"
    );

    for attr in packetizer.sdp_attributes() {
        sdp.push_str(&format!("a={attr}\r\n"));
    }

    sdp.push_str("a=control:track1\r\n");
    sdp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::h264::H264Packetizer;

    #[test]
    fn generates_h264_sdp() {
        let p = H264Packetizer::new(96, 0x12345678);
        let sdp = generate_sdp(&p);
        assert!(sdp.contains("v=0\r\n"));
        assert!(sdp.contains("a=rtpmap:96 H264/90000\r\n"));
        assert!(sdp.contains("a=fmtp:96 packetization-mode=1\r\n"));
        assert!(sdp.contains("a=control:track1\r\n"));
    }
}
