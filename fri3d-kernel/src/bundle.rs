//! `.fab` — Fri3d App Bundle.
//!
//! A bundle is a fixed 512-byte header followed by the payload (the app's
//! `.wasm`). Every field sits at a fixed offset so the kernel reads it in
//! place from flash; nothing is parsed or copied at boot. `fri3d-pack`
//! writes bundles from `manifest.toml` + `icon.png` + the compiled wasm.
//!
//! Layout (all integers little-endian):
//!
//! | Offset | Size | Field |
//! | ------ | ---- | ----- |
//! | 0      | 4    | magic `FAB1` |
//! | 4      | 2    | format version (2) |
//! | 6      | 2    | flags (bit 0 = system app) |
//! | 8      | 24   | id, `[a-z0-9_]`, NUL-padded |
//! | 32     | 32   | name |
//! | 64     | 16   | version |
//! | 80     | 32   | author |
//! | 112    | 96   | description |
//! | 208    | 1    | icon width (16) |
//! | 209    | 1    | icon height (16) |
//! | 210    | 1    | payload kind (0 = wasm) |
//! | 211    | 29   | reserved |
//! | 240    | 4    | payload length |
//! | 244    | 12   | reserved |
//! | 256    | 256  | icon, one palette index per pixel, row-major, 255 = transparent |
//! | 512    | n    | payload |

pub const MAGIC: &[u8; 4] = b"FAB1";
pub const FORMAT_VERSION: u16 = 2;
pub const HEADER_LEN: usize = 512;

pub const FLAG_SYSTEM: u16 = 1 << 0;

pub const PAYLOAD_WASM: u8 = 0;

pub const ICON_W: usize = 16;
pub const ICON_H: usize = 16;
pub const ICON_LEN: usize = ICON_W * ICON_H;
/// Icon pixel value that leaves the background untouched.
pub const ICON_TRANSPARENT: u8 = 255;

pub mod offset {
    pub const MAGIC: usize = 0;
    pub const FORMAT_VERSION: usize = 4;
    pub const FLAGS: usize = 6;
    pub const ID: usize = 8;
    pub const NAME: usize = 32;
    pub const VERSION: usize = 64;
    pub const AUTHOR: usize = 80;
    pub const DESCRIPTION: usize = 112;
    pub const ICON_W: usize = 208;
    pub const ICON_H: usize = 209;
    pub const PAYLOAD_KIND: usize = 210;
    pub const PAYLOAD_LEN: usize = 240;
    pub const ICON: usize = 256;
}

pub mod len {
    pub const ID: usize = 24;
    pub const NAME: usize = 32;
    pub const VERSION: usize = 16;
    pub const AUTHOR: usize = 32;
    pub const DESCRIPTION: usize = 96;
}

const _: () = assert!(offset::DESCRIPTION + len::DESCRIPTION <= offset::ICON_W);
const _: () = assert!(offset::PAYLOAD_LEN + 4 + 12 == offset::ICON);
const _: () = assert!(offset::ICON + ICON_LEN == HEADER_LEN);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BundleError {
    TooShort,
    BadMagic,
    UnsupportedVersion,
    BadIconSize,
    PayloadTruncated,
    UnsupportedPayload,
}

/// A validated view over bundle bytes. Copy-free.
#[derive(Copy, Clone, Debug)]
pub struct Bundle<'a> {
    bytes: &'a [u8],
}

impl<'a> Bundle<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, BundleError> {
        if bytes.len() < HEADER_LEN {
            return Err(BundleError::TooShort);
        }
        if &bytes[offset::MAGIC..offset::MAGIC + 4] != MAGIC {
            return Err(BundleError::BadMagic);
        }
        if read_u16(bytes, offset::FORMAT_VERSION) != FORMAT_VERSION {
            return Err(BundleError::UnsupportedVersion);
        }
        if bytes[offset::ICON_W] as usize != ICON_W || bytes[offset::ICON_H] as usize != ICON_H {
            return Err(BundleError::BadIconSize);
        }
        if bytes[offset::PAYLOAD_KIND] != PAYLOAD_WASM {
            return Err(BundleError::UnsupportedPayload);
        }
        let payload_len = read_u32(bytes, offset::PAYLOAD_LEN) as usize;
        if bytes.len() < HEADER_LEN + payload_len {
            return Err(BundleError::PayloadTruncated);
        }
        Ok(Self { bytes })
    }

    pub fn header(&self) -> &'a [u8] {
        &self.bytes[..HEADER_LEN]
    }

    pub fn payload(&self) -> &'a [u8] {
        let n = read_u32(self.bytes, offset::PAYLOAD_LEN) as usize;
        &self.bytes[HEADER_LEN..HEADER_LEN + n]
    }

    pub fn flags(&self) -> u16 {
        read_u16(self.bytes, offset::FLAGS)
    }

    pub fn is_system(&self) -> bool {
        self.flags() & FLAG_SYSTEM != 0
    }

    pub fn id(&self) -> &'a str {
        field_str(self.bytes, offset::ID, len::ID)
    }

    pub fn name(&self) -> &'a str {
        field_str(self.bytes, offset::NAME, len::NAME)
    }

    pub fn version(&self) -> &'a str {
        field_str(self.bytes, offset::VERSION, len::VERSION)
    }

    pub fn author(&self) -> &'a str {
        field_str(self.bytes, offset::AUTHOR, len::AUTHOR)
    }

    pub fn description(&self) -> &'a str {
        field_str(self.bytes, offset::DESCRIPTION, len::DESCRIPTION)
    }

    pub fn icon(&self) -> &'a [u8] {
        &self.bytes[offset::ICON..offset::ICON + ICON_LEN]
    }
}

/// Read a NUL-padded field as `&str`. Invalid UTF-8 yields an empty string
/// rather than a panic: the pack tool guarantees validity, the kernel does
/// not trust it.
pub fn field_str(bytes: &[u8], start: usize, max: usize) -> &str {
    let field = &bytes[start..start + max];
    let end = field.iter().position(|&b| b == 0).unwrap_or(max);
    core::str::from_utf8(&field[..end]).unwrap_or("")
}

pub fn read_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

pub fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// Build a header in place. Used by `fri3d-pack` and by tests; lives here
/// so writer and reader share one definition of the layout.
pub struct HeaderBuilder {
    buf: [u8; HEADER_LEN],
}

impl Default for HeaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HeaderBuilder {
    pub fn new() -> Self {
        let mut buf = [0u8; HEADER_LEN];
        buf[offset::MAGIC..offset::MAGIC + 4].copy_from_slice(MAGIC);
        buf[offset::FORMAT_VERSION..offset::FORMAT_VERSION + 2]
            .copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        buf[offset::ICON_W] = ICON_W as u8;
        buf[offset::ICON_H] = ICON_H as u8;
        buf[offset::PAYLOAD_KIND] = PAYLOAD_WASM;
        buf[offset::ICON..offset::ICON + ICON_LEN].fill(ICON_TRANSPARENT);
        Self { buf }
    }

    fn put_str(&mut self, at: usize, max: usize, value: &str) -> &mut Self {
        let bytes = value.as_bytes();
        // Leave room for at least one NUL so readers always terminate.
        let mut n = bytes.len().min(max - 1);
        while n > 0 && !value.is_char_boundary(n) {
            n -= 1;
        }
        self.buf[at..at + n].copy_from_slice(&bytes[..n]);
        self
    }

    pub fn id(&mut self, v: &str) -> &mut Self {
        self.put_str(offset::ID, len::ID, v)
    }
    pub fn name(&mut self, v: &str) -> &mut Self {
        self.put_str(offset::NAME, len::NAME, v)
    }
    pub fn version(&mut self, v: &str) -> &mut Self {
        self.put_str(offset::VERSION, len::VERSION, v)
    }
    pub fn author(&mut self, v: &str) -> &mut Self {
        self.put_str(offset::AUTHOR, len::AUTHOR, v)
    }
    pub fn description(&mut self, v: &str) -> &mut Self {
        self.put_str(offset::DESCRIPTION, len::DESCRIPTION, v)
    }
    pub fn flags(&mut self, flags: u16) -> &mut Self {
        self.buf[offset::FLAGS..offset::FLAGS + 2].copy_from_slice(&flags.to_le_bytes());
        self
    }
    pub fn icon(&mut self, pixels: &[u8; ICON_LEN]) -> &mut Self {
        self.buf[offset::ICON..offset::ICON + ICON_LEN].copy_from_slice(pixels);
        self
    }
    pub fn payload_len(&mut self, n: u32) -> &mut Self {
        self.buf[offset::PAYLOAD_LEN..offset::PAYLOAD_LEN + 4].copy_from_slice(&n.to_le_bytes());
        self
    }
    pub fn finish(&self) -> [u8; HEADER_LEN] {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn sample() -> Vec<u8> {
        let payload = b"\0asm\x01\0\0\0";
        let header = HeaderBuilder::new()
            .id("snake")
            .name("Snake")
            .version("0.1.0")
            .author("Fri3d")
            .description("Eat fruit")
            .flags(FLAG_SYSTEM)
            .payload_len(payload.len() as u32)
            .finish();
        let mut v = header.to_vec();
        v.extend_from_slice(payload);
        v
    }

    #[test]
    fn roundtrip() {
        let bytes = sample();
        let b = Bundle::parse(&bytes).unwrap();
        assert_eq!(b.id(), "snake");
        assert_eq!(b.name(), "Snake");
        assert_eq!(b.version(), "0.1.0");
        assert_eq!(b.author(), "Fri3d");
        assert_eq!(b.description(), "Eat fruit");
        assert!(b.is_system());
        assert_eq!(b.payload(), b"\0asm\x01\0\0\0");
    }

    #[test]
    fn rejects_truncated_payload() {
        let mut bytes = sample();
        bytes.truncate(HEADER_LEN + 2);
        assert_eq!(Bundle::parse(&bytes).unwrap_err(), BundleError::PayloadTruncated);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = sample();
        bytes[0] = b'X';
        assert_eq!(Bundle::parse(&bytes).unwrap_err(), BundleError::BadMagic);
    }

    #[test]
    fn long_strings_are_clipped_with_nul() {
        let long: alloc::string::String = "x".repeat(200);
        let header = HeaderBuilder::new().name(&long).finish();
        assert_eq!(field_str(&header, offset::NAME, len::NAME).len(), len::NAME - 1);
    }
}
