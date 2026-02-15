// TODO: MJPEG RTP packetizer — RFC 2435
//
// Simpler than H.264/H.265:
// - Each JPEG frame maps to one or more RTP packets
// - RTP payload starts with an 8-byte JPEG-specific header
// - No NAL unit concept — fragmentation is at the JPEG frame level
// - SDP: a=rtpmap:26 JPEG/90000 (static payload type 26)
//
// Good for IP cameras and low-latency preview streams.
