# RTSP Server

Publish live encoded video packets over RTSP from Rust, Python, or GStreamer. The library handles RTSP session negotiation, RTP packetization, and UDP delivery — you push encoded frames and any standard client (VLC, ffplay, etc.) can play the stream.

> **Note:** This project is still very new. The API is not stable yet and there may be breaking changes between releases.

## Project structure 📦

```
crates/
├── core/        # rtsp (library)
├── python/      # rtsp-python (PyO3 bindings)
├── gst/         # libgstrtspserversink (GStreamer plugin)
└── cli/         # rtsp-cli (standalone server binary)

examples/
└── python/      # Python demo (numpy + PyAV)
    ├── test.py
    └── requirements.txt
```

Use the package name with `cargo build -p <name>` (e.g. `-p rtsp`, `-p rtsp-cli`).

## Quick start 🚀

### Rust 🦀

```rust
use rtsp::Server;

let mut server = Server::new("0.0.0.0:8554");
server.start().unwrap();

let packetizer = rtsp::media::h264::H264Packetizer::with_random_ssrc(96);
let rtp_packets = packetizer.packetize(&h264_frame, 3000);
for packet in &rtp_packets {
    server.broadcast_rtp_packet(packet).unwrap();
}
```

### Python 🐍

```python
import rtsp

server = rtsp.Server("0.0.0.0:8554")
server.start()
packetizer = rtsp.H264Packetizer()
for packet in packetizer.packetize(h264_bytes, 3000):
    server.broadcast_rtp_packet(packet)
```

See **`examples/python/test.py`** for a full demo (numpy + PyAV) that streams a generated test pattern.

### GStreamer 🎬

Build the plugin, then stream:

```bash
cargo build -p gst-rtsp-sink --release
# Copy target/release/libgstrtspserversink.so to your GStreamer plugin path, or set GST_PLUGIN_PATH

gst-launch-1.0 videotestsrc ! x264enc tune=zerolatency \
    ! 'video/x-h264,stream-format=byte-stream' ! rtspserversink port=8554
```

Then open in VLC: `rtsp://localhost:8554/stream` ✨

### CLI server 🖥️

```bash
cargo build -p rtsp-cli --release
./target/release/rtsp-server --bind 0.0.0.0:8554
```

## Building 🔧

Rust 1.85+.

```bash
cargo build -p rtsp              # Core
cargo build -p rtsp-python       # Python (needs maturin)
cargo build -p gst-rtsp-server-sink     # GStreamer (needs libgstreamer1.0-dev)
cargo build -p rtsp-cli          # CLI server
cargo test --workspace
```
