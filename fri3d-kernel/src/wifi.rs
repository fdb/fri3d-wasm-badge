//! Wi-Fi: the kernel owns the model, hosts own the radio.
//!
//! The model is the saved-network store, the last scan, the link state
//! and the auto-connect state machine. Hosts execute three primitives
//! (`WifiRequest::Scan`, `Connect`, `Disconnect`) and report back with
//! `scan_done`, `connect_done` and `link_lost`. All policy — which saved
//! network to try first, what to do when one fails, when to retry — lives
//! here, so it is identical on the badge, the desktop and the browser,
//! and testable without a radio.
//!
//! Passwords never leave the kernel: apps can list saved SSIDs and ask to
//! connect, but only the host driver receives the password, inside a
//! `WifiRequest::Connect`.

use crate::limits::{WIFI_SAVED_MAX, WIFI_SCAN_MAX};
use heapless::{String, Vec};

pub const SSID_LEN: usize = 32;
pub const PASSWORD_LEN: usize = 64;
pub type Ssid = String<SSID_LEN>;
pub type Password = String<PASSWORD_LEN>;

/// Persistent image: magic, count, then `WIFI_SAVED_MAX` fixed entries
/// (SSID padded to 32 bytes, password padded to 64).
const ENTRY_LEN: usize = SSID_LEN + PASSWORD_LEN;
pub const IMAGE_LEN: usize = 8 + WIFI_SAVED_MAX * ENTRY_LEN;
const IMAGE_MAGIC: &[u8; 4] = b"FWF1";

/// Time after an unexpected link loss before auto-connect tries again.
const RECONNECT_DELAY_MS: u32 = 10_000;

/// Link state as apps see it. Scanning is reported separately because it
/// can overlap with `Connected`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum WifiStatus {
    /// Radio disabled by the user (`system.wifi = 0`).
    Off = 0,
    /// Enabled, not connected, nothing in progress.
    Idle = 1,
    /// A connect is in flight for `current_ssid`.
    Connecting = 2,
    /// Associated with `current_ssid`.
    Connected = 3,
    /// The last attempt (or every auto-connect candidate) failed.
    Failed = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanEntry {
    pub ssid: Ssid,
    pub rssi: i8,
    pub secure: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedNetwork {
    pub ssid: Ssid,
    pub password: Password,
}

/// What the host driver must do next. One slot: a newer request replaces
/// an unserviced older one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WifiRequest {
    Scan,
    Connect { ssid: Ssid, password: Password },
    Disconnect,
}

pub struct Wifi {
    enabled: bool,
    status: WifiStatus,
    scanning: bool,
    current: Ssid,
    scan: Vec<ScanEntry, WIFI_SCAN_MAX>,
    saved: Vec<SavedNetwork, WIFI_SAVED_MAX>,
    request: Option<WifiRequest>,
    /// The in-flight scan/connect belongs to auto-connect, which walks the
    /// saved networks strongest-first. Manual actions clear it.
    auto: bool,
    /// Saved-network indices auto-connect already tried this round.
    tried: u8,
    retry_at: Option<u32>,
    dirty: bool,
    changed: bool,
}

impl Default for Wifi {
    fn default() -> Self {
        Self::new()
    }
}

impl Wifi {
    pub const fn new() -> Self {
        Self {
            enabled: false,
            status: WifiStatus::Off,
            scanning: false,
            current: String::new(),
            scan: Vec::new(),
            saved: Vec::new(),
            request: None,
            auto: false,
            tried: 0,
            retry_at: None,
            dirty: false,
            changed: false,
        }
    }

    // -- state for apps and hosts ------------------------------------------

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn status(&self) -> WifiStatus {
        self.status
    }

    pub fn scanning(&self) -> bool {
        self.scanning
    }

    /// SSID of the network being connected to, connected to, or that
    /// failed last. Empty when idle.
    pub fn current_ssid(&self) -> &str {
        &self.current
    }

    pub fn scan_results(&self) -> &[ScanEntry] {
        &self.scan
    }

    pub fn saved(&self) -> &[SavedNetwork] {
        &self.saved
    }

    pub fn saved_index(&self, ssid: &str) -> Option<usize> {
        self.saved.iter().position(|n| n.ssid.as_str() == ssid)
    }

    // -- app actions --------------------------------------------------------

    /// Turn the radio on or off. On: start auto-connect. Off: drop the
    /// link, forget the scan.
    pub fn set_enabled(&mut self, on: bool) {
        if self.enabled == on {
            return;
        }
        self.enabled = on;
        self.retry_at = None;
        self.auto = false;
        if on {
            self.set_status(WifiStatus::Idle);
            self.start_auto();
        } else {
            self.scan.clear();
            self.scanning = false;
            self.current.clear();
            self.set_status(WifiStatus::Off);
            self.set_request(WifiRequest::Disconnect);
        }
    }

    /// Scan on the user's behalf. False when the radio is off.
    pub fn scan(&mut self) -> bool {
        if !self.enabled || self.scanning {
            return self.enabled;
        }
        self.auto = false;
        self.scanning = true;
        self.changed = true;
        self.set_request(WifiRequest::Scan);
        true
    }

    /// Connect to a saved network now. False when unknown or radio off.
    pub fn connect(&mut self, ssid: &str) -> bool {
        if !self.enabled {
            return false;
        }
        let Some(i) = self.saved_index(ssid) else { return false };
        self.auto = false;
        self.retry_at = None;
        self.begin_connect(i);
        true
    }

    pub fn disconnect(&mut self) {
        if !self.enabled {
            return;
        }
        self.auto = false;
        self.retry_at = None;
        self.current.clear();
        self.set_status(WifiStatus::Idle);
        self.set_request(WifiRequest::Disconnect);
    }

    /// Add or update a network. False when the store is full or the SSID
    /// is empty.
    pub fn save(&mut self, ssid: &str, password: &str) -> bool {
        if ssid.is_empty() || ssid.len() > SSID_LEN || password.len() > PASSWORD_LEN {
            return false;
        }
        let mut pw: Password = String::new();
        let _ = pw.push_str(password);
        if let Some(i) = self.saved_index(ssid) {
            if self.saved[i].password != pw {
                self.saved[i].password = pw;
                self.dirty = true;
            }
            return true;
        }
        let mut s: Ssid = String::new();
        let _ = s.push_str(ssid);
        if self.saved.push(SavedNetwork { ssid: s, password: pw }).is_err() {
            return false;
        }
        self.dirty = true;
        self.changed = true;
        true
    }

    pub fn forget(&mut self, ssid: &str) -> bool {
        let Some(i) = self.saved_index(ssid) else { return false };
        self.saved.remove(i);
        self.dirty = true;
        self.changed = true;
        if self.current.as_str() == ssid
            && matches!(self.status, WifiStatus::Connected | WifiStatus::Connecting)
        {
            self.disconnect();
        }
        true
    }

    // -- host driver ------------------------------------------------------

    pub fn take_request(&mut self) -> Option<WifiRequest> {
        self.request.take()
    }

    pub fn scan_done(&mut self, entries: &[ScanEntry]) {
        self.scanning = false;
        self.scan.clear();
        for e in entries.iter().take(WIFI_SCAN_MAX) {
            let _ = self.scan.push(e.clone());
        }
        // Strongest first; that is also the auto-connect order.
        sort_by_rssi(&mut self.scan);
        self.changed = true;
        if self.auto {
            self.try_next();
        }
    }

    pub fn connect_done(&mut self, ok: bool) {
        if self.status != WifiStatus::Connecting {
            return;
        }
        if ok {
            self.auto = false;
            self.tried = 0;
            self.set_status(WifiStatus::Connected);
        } else if self.auto {
            self.try_next();
        } else {
            self.set_status(WifiStatus::Failed);
        }
    }

    /// The host noticed the association dropped. Auto-connect retries
    /// after a pause, so a rebooted access point comes back on its own.
    pub fn link_lost(&mut self, now_ms: u32) {
        if self.status != WifiStatus::Connected {
            return;
        }
        self.set_status(WifiStatus::Idle);
        self.retry_at = Some(now_ms.wrapping_add(RECONNECT_DELAY_MS));
    }

    /// Called once per kernel step.
    pub fn tick(&mut self, now_ms: u32) {
        if let Some(at) = self.retry_at {
            if (now_ms.wrapping_sub(at) as i32) >= 0 {
                self.retry_at = None;
                self.start_auto();
            }
        }
    }

    /// True once when anything an app could display changed.
    pub fn take_changed(&mut self) -> bool {
        core::mem::replace(&mut self.changed, false)
    }

    // -- persistence ----------------------------------------------------------

    pub fn take_dirty(&mut self) -> bool {
        core::mem::replace(&mut self.dirty, false)
    }

    pub fn write_image(&self, out: &mut [u8; IMAGE_LEN]) {
        out.fill(0);
        out[..4].copy_from_slice(IMAGE_MAGIC);
        out[4..8].copy_from_slice(&(self.saved.len() as u32).to_le_bytes());
        for (i, n) in self.saved.iter().enumerate() {
            let at = 8 + i * ENTRY_LEN;
            out[at..at + n.ssid.len()].copy_from_slice(n.ssid.as_bytes());
            let pw = at + SSID_LEN;
            out[pw..pw + n.password.len()].copy_from_slice(n.password.as_bytes());
        }
    }

    /// Ignores garbage, like the settings image.
    pub fn load_image(&mut self, image: &[u8]) {
        self.saved.clear();
        if image.len() < 8 || &image[..4] != IMAGE_MAGIC {
            return;
        }
        let count = u32::from_le_bytes([image[4], image[5], image[6], image[7]]) as usize;
        let body = &image[8..];
        for chunk in body.as_chunks::<ENTRY_LEN>().0.iter().take(count.min(WIFI_SAVED_MAX)) {
            let (Some(ssid), Some(password)) = (
                padded_str::<SSID_LEN>(&chunk[..SSID_LEN]),
                padded_str::<PASSWORD_LEN>(&chunk[SSID_LEN..]),
            ) else {
                continue;
            };
            if ssid.is_empty() {
                continue;
            }
            let _ = self.saved.push(SavedNetwork { ssid, password });
        }
        self.dirty = false;
    }

    // -- internals ------------------------------------------------------------

    fn set_status(&mut self, s: WifiStatus) {
        if self.status != s {
            self.status = s;
            self.changed = true;
        }
    }

    fn set_request(&mut self, r: WifiRequest) {
        // Dropping an unserviced request must not leave its state behind.
        match self.request.take() {
            Some(WifiRequest::Scan) if r != WifiRequest::Scan => self.scanning = false,
            _ => {}
        }
        self.request = Some(r);
    }

    /// Start the auto-connect round: scan, then walk saved networks.
    pub fn start_auto(&mut self) {
        if !self.enabled || self.saved.is_empty() || self.status == WifiStatus::Connected {
            return;
        }
        self.auto = true;
        self.tried = 0;
        self.scanning = true;
        self.changed = true;
        self.set_request(WifiRequest::Scan);
    }

    fn begin_connect(&mut self, saved_index: usize) {
        self.current.clear();
        let _ = self.current.push_str(&self.saved[saved_index].ssid);
        self.set_status(WifiStatus::Connecting);
        let n = &self.saved[saved_index];
        self.request = Some(WifiRequest::Connect {
            ssid: n.ssid.clone(),
            password: n.password.clone(),
        });
    }

    /// Next auto-connect candidate: the strongest visible saved network
    /// not yet tried this round. None left: report failure (or idle when
    /// nothing was visible at all).
    fn try_next(&mut self) {
        let candidate = self
            .scan
            .iter()
            .filter_map(|e| self.saved_index(&e.ssid))
            .find(|&i| self.tried & (1 << i) == 0);
        match candidate {
            Some(i) => {
                self.tried |= 1 << i;
                self.begin_connect(i);
            }
            None => {
                self.auto = false;
                let status = if self.tried == 0 { WifiStatus::Idle } else { WifiStatus::Failed };
                if status == WifiStatus::Idle {
                    self.current.clear();
                }
                self.set_status(status);
            }
        }
    }
}

fn sort_by_rssi(v: &mut Vec<ScanEntry, WIFI_SCAN_MAX>) {
    // Insertion sort: at most 16 entries, no allocation.
    for i in 1..v.len() {
        let mut j = i;
        while j > 0 && v[j - 1].rssi < v[j].rssi {
            v.swap(j - 1, j);
            j -= 1;
        }
    }
}

fn padded_str<const N: usize>(bytes: &[u8]) -> Option<String<N>> {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let s = core::str::from_utf8(&bytes[..len]).ok()?;
    let mut out = String::new();
    out.push_str(s).ok()?;
    Some(out)
}

// ---------------------------------------------------------------------------
// Simulated radio
// ---------------------------------------------------------------------------

/// A deterministic fake driver for the desktop host, the browser and the
/// kernel tests. Scans take 1.5 s, connects 2 s, and only the right
/// password (or an open network) succeeds.
pub struct Sim {
    pending: Option<(u32, SimOp)>,
}

enum SimOp {
    Scan,
    Connect(bool),
}

/// (ssid, rssi, password; None = open network)
pub const SIM_NETWORKS: &[(&str, i8, Option<&str>)] = &[
    ("Fri3d Camp", -45, Some("fri3d2026")),
    ("Hackerspace", -60, Some("hunter22")),
    ("Free WiFi", -72, None),
    ("Neighbour", -85, Some("nope")),
];

const SIM_SCAN_MS: u32 = 1500;
const SIM_CONNECT_MS: u32 = 2000;

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

impl Sim {
    pub const fn new() -> Self {
        Self { pending: None }
    }

    /// Drive one request slot. Call every host iteration with the kernel's
    /// clock.
    pub fn service(&mut self, wifi: &mut Wifi, now_ms: u32) {
        if let Some((done_at, _)) = &self.pending {
            if (now_ms.wrapping_sub(*done_at) as i32) < 0 {
                return;
            }
            match self.pending.take().unwrap().1 {
                SimOp::Scan => {
                    let mut entries: Vec<ScanEntry, WIFI_SCAN_MAX> = Vec::new();
                    for (ssid, rssi, pw) in SIM_NETWORKS {
                        let mut s: Ssid = String::new();
                        let _ = s.push_str(ssid);
                        let _ = entries.push(ScanEntry { ssid: s, rssi: *rssi, secure: pw.is_some() });
                    }
                    wifi.scan_done(&entries);
                }
                SimOp::Connect(ok) => wifi.connect_done(ok),
            }
        }
        match wifi.take_request() {
            None => {}
            Some(WifiRequest::Scan) => self.pending = Some((now_ms + SIM_SCAN_MS, SimOp::Scan)),
            Some(WifiRequest::Connect { ssid, password }) => {
                let ok = SIM_NETWORKS
                    .iter()
                    .any(|(s, _, pw)| *s == ssid.as_str() && pw.is_none_or(|p| p == password.as_str()));
                self.pending = Some((now_ms + SIM_CONNECT_MS, SimOp::Connect(ok)));
            }
            Some(WifiRequest::Disconnect) => self.pending = None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(ssid: &str, rssi: i8) -> ScanEntry {
        let mut s: Ssid = String::new();
        s.push_str(ssid).unwrap();
        ScanEntry { ssid: s, rssi, secure: true }
    }

    #[test]
    fn saved_networks_roundtrip_through_image() {
        let mut w = Wifi::new();
        assert!(w.save("Fri3d Camp", "fri3d2026"));
        assert!(w.save("Open", ""));
        assert!(w.take_dirty());
        let mut img = [0u8; IMAGE_LEN];
        w.write_image(&mut img);
        let mut t = Wifi::new();
        t.load_image(&img);
        assert_eq!(t.saved(), w.saved());
        assert!(!t.take_dirty());
    }

    #[test]
    fn garbage_image_is_ignored() {
        let mut w = Wifi::new();
        w.load_image(b"nonsense");
        assert!(w.saved().is_empty());
    }

    #[test]
    fn auto_connect_walks_saved_networks_strongest_first() {
        let mut w = Wifi::new();
        w.save("Weak", "a");
        w.save("Strong", "b");
        w.set_enabled(true);
        assert_eq!(w.take_request(), Some(WifiRequest::Scan));
        assert!(w.scanning());
        w.scan_done(&[entry("Weak", -80), entry("Strong", -40), entry("Unknown", -30)]);
        assert!(!w.scanning());
        assert_eq!(w.status(), WifiStatus::Connecting);
        assert_eq!(w.current_ssid(), "Strong");
        assert!(matches!(w.take_request(), Some(WifiRequest::Connect { ssid, password }) if ssid == "Strong" && password == "b"));
        w.connect_done(false);
        assert_eq!(w.current_ssid(), "Weak");
        assert!(matches!(w.take_request(), Some(WifiRequest::Connect { .. })));
        w.connect_done(false);
        assert_eq!(w.status(), WifiStatus::Failed);
        assert_eq!(w.take_request(), None);
    }

    #[test]
    fn auto_connect_idle_when_no_saved_network_is_visible() {
        let mut w = Wifi::new();
        w.save("Home", "pw");
        w.set_enabled(true);
        w.take_request();
        w.scan_done(&[entry("Other", -50)]);
        assert_eq!(w.status(), WifiStatus::Idle);
        assert_eq!(w.current_ssid(), "");
    }

    #[test]
    fn manual_connect_failure_does_not_fall_through_to_others() {
        let mut w = Wifi::new();
        w.save("A", "1");
        w.save("B", "2");
        w.set_enabled(true);
        w.take_request();
        w.scan_done(&[entry("A", -50), entry("B", -60)]);
        w.take_request();
        w.connect_done(true);
        assert_eq!(w.status(), WifiStatus::Connected);
        assert!(w.connect("B"));
        w.take_request();
        w.connect_done(false);
        assert_eq!(w.status(), WifiStatus::Failed);
        assert_eq!(w.current_ssid(), "B");
        assert_eq!(w.take_request(), None);
    }

    #[test]
    fn disabling_drops_the_link_and_blocks_actions() {
        let mut w = Wifi::new();
        w.save("A", "1");
        w.set_enabled(true);
        w.take_request();
        w.scan_done(&[entry("A", -50)]);
        w.take_request();
        w.connect_done(true);
        w.set_enabled(false);
        assert_eq!(w.status(), WifiStatus::Off);
        assert_eq!(w.take_request(), Some(WifiRequest::Disconnect));
        assert!(!w.scan());
        assert!(!w.connect("A"));
        assert!(w.scan_results().is_empty());
    }

    #[test]
    fn link_loss_retries_after_delay() {
        let mut w = Wifi::new();
        w.save("A", "1");
        w.set_enabled(true);
        w.take_request();
        w.scan_done(&[entry("A", -50)]);
        w.take_request();
        w.connect_done(true);
        w.link_lost(1000);
        assert_eq!(w.status(), WifiStatus::Idle);
        w.tick(5000);
        assert_eq!(w.take_request(), None);
        w.tick(11_000);
        assert_eq!(w.take_request(), Some(WifiRequest::Scan));
    }

    #[test]
    fn forgetting_the_connected_network_disconnects() {
        let mut w = Wifi::new();
        w.save("A", "1");
        w.set_enabled(true);
        w.take_request();
        w.scan_done(&[entry("A", -50)]);
        w.take_request();
        w.connect_done(true);
        assert!(w.forget("A"));
        assert_eq!(w.status(), WifiStatus::Idle);
        assert_eq!(w.take_request(), Some(WifiRequest::Disconnect));
        assert!(w.saved().is_empty());
    }

    #[test]
    fn store_is_bounded() {
        let mut w = Wifi::new();
        for i in 0..WIFI_SAVED_MAX {
            let mut s: String<8> = String::new();
            use core::fmt::Write;
            write!(s, "n{i}").unwrap();
            assert!(w.save(&s, "pw"));
        }
        assert!(!w.save("overflow", "pw"));
        assert!(!w.save("", "pw"));
    }

    #[test]
    fn sim_connects_with_the_right_password_only() {
        let mut w = Wifi::new();
        let mut sim = Sim::new();
        w.save("Fri3d Camp", "wrong");
        w.set_enabled(true);
        let mut t = 0;
        while t < 10_000 {
            sim.service(&mut w, t);
            t += 100;
        }
        assert_eq!(w.status(), WifiStatus::Failed);
        w.save("Fri3d Camp", "fri3d2026");
        assert!(w.connect("Fri3d Camp"));
        while t < 20_000 {
            sim.service(&mut w, t);
            t += 100;
        }
        assert_eq!(w.status(), WifiStatus::Connected);
        assert_eq!(w.scan_results().len(), SIM_NETWORKS.len());
        assert_eq!(w.scan_results()[0].ssid, "Fri3d Camp");
    }
}
