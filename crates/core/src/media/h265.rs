// TODO: H.265 (HEVC) RTP packetizer — RFC 7798
//
// Key differences from H.264:
// - 2-byte NAL unit header (vs 1-byte in H.264)
// - FU header format: FU indicator (1 byte) + FU header (1 byte) with 6-bit NAL type
// - SDP attributes: a=rtpmap:96 H265/90000
//   and a=fmtp:96 sprop-vps=...; sprop-sps=...; sprop-pps=...
//
// Implementation will follow the same pattern as H264Packetizer:
// - Compose an RtpHeader for generic header building
// - Implement the Packetizer trait
// - Extract NAL units from Annex B (same start codes, different NAL header parsing)
