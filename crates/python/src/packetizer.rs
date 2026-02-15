use std::sync::{Arc, Mutex};

use pyo3::{exceptions::PyRuntimeError, prelude::*};

use rtsp::Packetizer;
use rtsp::media::h264::H264Packetizer;

#[pyclass(name = "H264Packetizer")]
pub struct PyH264Packetizer {
    inner: Arc<Mutex<H264Packetizer>>,
}

#[pymethods]
impl PyH264Packetizer {
    #[new]
    #[pyo3(signature = (pt = 96))]
    fn new(pt: u8) -> Self {
        PyH264Packetizer {
            inner: Arc::new(Mutex::new(H264Packetizer::with_random_ssrc(pt))),
        }
    }

    fn packetize(&self, frame_data: &[u8], timestamp_increment: u32) -> PyResult<Vec<Vec<u8>>> {
        Ok(self
            .inner
            .lock()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?
            .packetize(frame_data, timestamp_increment))
    }
}
