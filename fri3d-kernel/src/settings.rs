//! Settings: a fixed table of `(namespace, key) -> u32`.
//!
//! Namespaces are app ids; `system` is reserved for kernel-wide values
//! (brightness, sound) and writable only by apps with the system flag.
//! The table serializes to a flat byte image so hosts can persist it
//! without knowing the layout.

use crate::limits::SETTINGS_ENTRIES;

pub const NS_LEN: usize = 24;
pub const KEY_LEN: usize = 24;
pub const ENTRY_LEN: usize = NS_LEN + KEY_LEN + 4;
pub const IMAGE_LEN: usize = 4 + SETTINGS_ENTRIES * ENTRY_LEN;

const IMAGE_MAGIC: &[u8; 4] = b"FST1";

pub const SYSTEM_NS: &str = "system";

#[derive(Copy, Clone)]
struct Entry {
    ns: [u8; NS_LEN],
    key: [u8; KEY_LEN],
    value: u32,
}

const EMPTY: Entry = Entry {
    ns: [0; NS_LEN],
    key: [0; KEY_LEN],
    value: 0,
};

pub struct Settings {
    entries: [Entry; SETTINGS_ENTRIES],
    used: usize,
    dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            entries: [EMPTY; SETTINGS_ENTRIES],
            used: 0,
            dirty: false,
        }
    }

    pub fn get(&self, ns: &str, key: &str) -> Option<u32> {
        let (ns, key) = (pad::<NS_LEN>(ns), pad::<KEY_LEN>(key));
        self.entries[..self.used]
            .iter()
            .find(|e| e.ns == ns && e.key == key)
            .map(|e| e.value)
    }

    pub fn get_or(&self, ns: &str, key: &str, default: u32) -> u32 {
        self.get(ns, key).unwrap_or(default)
    }

    /// Returns false when the table is full.
    pub fn set(&mut self, ns: &str, key: &str, value: u32) -> bool {
        let (ns, key) = (pad::<NS_LEN>(ns), pad::<KEY_LEN>(key));
        if let Some(e) = self.entries[..self.used]
            .iter_mut()
            .find(|e| e.ns == ns && e.key == key)
        {
            if e.value != value {
                e.value = value;
                self.dirty = true;
            }
            return true;
        }
        if self.used == SETTINGS_ENTRIES {
            return false;
        }
        self.entries[self.used] = Entry { ns, key, value };
        self.used += 1;
        self.dirty = true;
        true
    }

    /// True if anything changed since the last `take_dirty`.
    pub fn take_dirty(&mut self) -> bool {
        core::mem::replace(&mut self.dirty, false)
    }

    pub fn write_image(&self, out: &mut [u8; IMAGE_LEN]) {
        out.fill(0);
        out[..4].copy_from_slice(IMAGE_MAGIC);
        for (i, e) in self.entries[..self.used].iter().enumerate() {
            let at = 4 + i * ENTRY_LEN;
            out[at..at + NS_LEN].copy_from_slice(&e.ns);
            out[at + NS_LEN..at + NS_LEN + KEY_LEN].copy_from_slice(&e.key);
            out[at + NS_LEN + KEY_LEN..at + ENTRY_LEN].copy_from_slice(&e.value.to_le_bytes());
        }
    }

    /// Load a previously written image. Ignores garbage: a corrupt
    /// settings blob must never brick the badge.
    pub fn load_image(&mut self, image: &[u8]) {
        *self = Self::new();
        if image.len() < 4 || &image[..4] != IMAGE_MAGIC {
            return;
        }
        let body = &image[4..];
        for chunk in body.as_chunks::<ENTRY_LEN>().0.iter().take(SETTINGS_ENTRIES) {
            let mut ns = [0u8; NS_LEN];
            let mut key = [0u8; KEY_LEN];
            ns.copy_from_slice(&chunk[..NS_LEN]);
            key.copy_from_slice(&chunk[NS_LEN..NS_LEN + KEY_LEN]);
            if ns[0] == 0 {
                break;
            }
            let value = u32::from_le_bytes([
                chunk[NS_LEN + KEY_LEN],
                chunk[NS_LEN + KEY_LEN + 1],
                chunk[NS_LEN + KEY_LEN + 2],
                chunk[NS_LEN + KEY_LEN + 3],
            ]);
            self.entries[self.used] = Entry { ns, key, value };
            self.used += 1;
        }
        self.dirty = false;
    }
}

fn pad<const N: usize>(s: &str) -> [u8; N] {
    let mut out = [0u8; N];
    let bytes = s.as_bytes();
    let n = bytes.len().min(N);
    out[..n].copy_from_slice(&bytes[..n]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_roundtrip_through_image() {
        let mut s = Settings::new();
        assert!(s.set("system", "brightness", 80));
        assert!(s.set("snake", "highscore", 42));
        assert!(s.take_dirty());
        let mut img = [0u8; IMAGE_LEN];
        s.write_image(&mut img);
        let mut t = Settings::new();
        t.load_image(&img);
        assert_eq!(t.get("system", "brightness"), Some(80));
        assert_eq!(t.get("snake", "highscore"), Some(42));
        assert_eq!(t.get("snake", "missing"), None);
    }

    #[test]
    fn full_table_rejects() {
        let mut s = Settings::new();
        for i in 0..SETTINGS_ENTRIES {
            let mut k = heapless::String::<8>::new();
            use core::fmt::Write;
            write!(k, "k{i}").unwrap();
            assert!(s.set("a", &k, 0));
        }
        assert!(!s.set("a", "overflow", 0));
    }

    #[test]
    fn garbage_image_is_ignored() {
        let mut s = Settings::new();
        s.load_image(b"nope");
        assert_eq!(s.get("system", "brightness"), None);
    }
}
