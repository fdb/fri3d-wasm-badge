//! Network operations: one at a time, host-executed, progress-only.
//!
//! Apps never see sockets or payload bytes. They start an operation, the
//! host runs it (desktop: std sockets; badge: embassy-net; browser: the
//! simulator) and reports progress; the app polls `status`, `bytes` and
//! the elapsed time. Two primitives cover a connectivity check and a
//! throughput measurement:
//!
//! - `Probe { ip, port }`: open a TCP connection and close it.
//! - `Download { url }`: HTTP GET, count body bytes, discard them.
//!
//! Stopping the app that started an operation cancels it.

use heapless::String;

pub const URL_LEN: usize = 96;
pub type Url = String<URL_LEN>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum NetStatus {
    Idle = 0,
    Busy = 1,
    Done = 2,
    Failed = 3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetRequest {
    Probe { ip: [u8; 4], port: u16 },
    Download { url: Url },
    /// Abort whatever is in flight. The host reports nothing afterwards.
    Cancel,
}

/// Progress is flagged as a visible change at most every this many bytes,
/// so a fast download does not force a frame per packet.
const PROGRESS_STEP: u32 = 64 * 1024;

pub struct Net {
    request: Option<NetRequest>,
    status: NetStatus,
    bytes: u32,
    started_ms: u32,
    ended_ms: u32,
    last_reported: u32,
    changed: bool,
}

impl Default for Net {
    fn default() -> Self {
        Self::new()
    }
}

impl Net {
    pub const fn new() -> Self {
        Self {
            request: None,
            status: NetStatus::Idle,
            bytes: 0,
            started_ms: 0,
            ended_ms: 0,
            last_reported: 0,
            changed: false,
        }
    }

    pub fn status(&self) -> NetStatus {
        self.status
    }

    pub fn bytes(&self) -> u32 {
        self.bytes
    }

    /// Milliseconds the current or last operation took (so far).
    pub fn elapsed_ms(&self, now_ms: u32) -> u32 {
        match self.status {
            NetStatus::Idle => 0,
            NetStatus::Busy => now_ms.wrapping_sub(self.started_ms),
            _ => self.ended_ms.wrapping_sub(self.started_ms),
        }
    }

    // -- app side ---------------------------------------------------------

    /// False while another operation is in flight.
    pub fn probe(&mut self, ip: [u8; 4], port: u16, now_ms: u32) -> bool {
        self.start(NetRequest::Probe { ip, port }, now_ms)
    }

    pub fn download(&mut self, url: &str, now_ms: u32) -> bool {
        let mut u: Url = String::new();
        if url.is_empty() || u.push_str(url).is_err() {
            return false;
        }
        self.start(NetRequest::Download { url: u }, now_ms)
    }

    pub fn cancel(&mut self) {
        match (&self.request, self.status) {
            // Never picked up by the host: nothing to abort.
            (Some(NetRequest::Probe { .. }) | Some(NetRequest::Download { .. }), _) => {
                self.request = None;
                self.status = NetStatus::Idle;
            }
            (_, NetStatus::Busy) => {
                self.request = Some(NetRequest::Cancel);
                self.status = NetStatus::Idle;
                self.changed = true;
            }
            _ => {}
        }
    }

    fn start(&mut self, r: NetRequest, now_ms: u32) -> bool {
        if self.status == NetStatus::Busy || self.request.is_some() {
            return false;
        }
        self.request = Some(r);
        self.status = NetStatus::Busy;
        self.bytes = 0;
        self.last_reported = 0;
        self.started_ms = now_ms;
        self.changed = true;
        true
    }

    // -- host side ------------------------------------------------------

    pub fn take_request(&mut self) -> Option<NetRequest> {
        self.request.take()
    }

    pub fn progress(&mut self, bytes: u32) {
        if self.status != NetStatus::Busy {
            return;
        }
        self.bytes = bytes;
        if bytes.wrapping_sub(self.last_reported) >= PROGRESS_STEP {
            self.last_reported = bytes;
            self.changed = true;
        }
    }

    pub fn done(&mut self, ok: bool, now_ms: u32) {
        if self.status != NetStatus::Busy {
            return;
        }
        self.status = if ok { NetStatus::Done } else { NetStatus::Failed };
        self.ended_ms = now_ms;
        self.changed = true;
    }

    pub fn take_changed(&mut self) -> bool {
        core::mem::replace(&mut self.changed, false)
    }
}

/// Deterministic fake for the browser host and tests: probes succeed in
/// 30 ms, downloads deliver 1 MB at 2 MB/s.
pub struct Sim {
    op: Option<(u32, bool)>, // (started_ms, is_download)
}

pub const SIM_DOWNLOAD_BYTES: u32 = 1024 * 1024;
const SIM_BYTES_PER_MS: u32 = 2 * 1024;
const SIM_PROBE_MS: u32 = 30;

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

impl Sim {
    pub const fn new() -> Self {
        Self { op: None }
    }

    pub fn service(&mut self, net: &mut Net, now_ms: u32) {
        match net.take_request() {
            Some(NetRequest::Cancel) => self.op = None,
            Some(NetRequest::Probe { .. }) => self.op = Some((now_ms, false)),
            Some(NetRequest::Download { .. }) => self.op = Some((now_ms, true)),
            None => {}
        }
        let Some((t0, download)) = self.op else { return };
        let dt = now_ms.wrapping_sub(t0);
        if download {
            let bytes = dt.saturating_mul(SIM_BYTES_PER_MS).min(SIM_DOWNLOAD_BYTES);
            net.progress(bytes);
            if bytes == SIM_DOWNLOAD_BYTES {
                net.done(true, now_ms);
                self.op = None;
            }
        } else if dt >= SIM_PROBE_MS {
            net.done(true, now_ms);
            self.op = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_operation_at_a_time() {
        let mut n = Net::new();
        assert!(n.probe([1, 1, 1, 1], 53, 0));
        assert!(!n.download("http://x/y", 0));
        assert_eq!(n.take_request(), Some(NetRequest::Probe { ip: [1, 1, 1, 1], port: 53 }));
        n.done(true, 25);
        assert_eq!(n.status(), NetStatus::Done);
        assert_eq!(n.elapsed_ms(1000), 25);
        assert!(n.download("http://x/y", 1000));
    }

    #[test]
    fn cancel_reaches_the_host_only_when_in_flight() {
        let mut n = Net::new();
        n.probe([8, 8, 8, 8], 53, 0);
        n.cancel();
        assert_eq!(n.take_request(), None, "never started: nothing to abort");
        n.probe([8, 8, 8, 8], 53, 0);
        n.take_request();
        n.cancel();
        assert_eq!(n.take_request(), Some(NetRequest::Cancel));
        assert_eq!(n.status(), NetStatus::Idle);
        n.done(true, 5);
        assert_eq!(n.status(), NetStatus::Idle, "late report after cancel is ignored");
    }

    #[test]
    fn progress_throttles_visible_changes() {
        let mut n = Net::new();
        n.download("http://x/y", 0);
        n.take_request();
        assert!(n.take_changed());
        n.progress(1000);
        assert!(!n.take_changed());
        n.progress(70_000);
        assert!(n.take_changed());
        assert_eq!(n.bytes(), 70_000);
    }

    #[test]
    fn sim_completes_a_download() {
        let mut n = Net::new();
        let mut sim = Sim::new();
        n.download("http://speedtest/10MB", 0);
        let mut t = 0;
        while n.status() == NetStatus::Busy && t < 20_000 {
            sim.service(&mut n, t);
            t += 10;
        }
        assert_eq!(n.status(), NetStatus::Done);
        assert_eq!(n.bytes(), SIM_DOWNLOAD_BYTES);
        assert_eq!(n.elapsed_ms(t), 520);
    }
}
