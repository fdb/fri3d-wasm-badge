# Stage 013: Storage (Internal Settings + External SD)

This spec defines a two-tier storage model for the ESP32 target:
- **Internal settings storage** for small, critical data (brightness, WiFi creds, highscores).
- **External SD storage** for apps, files, and media.

The API and path mapping are designed to mirror Flipper Zero concepts while
fitting the Fri3d runtime and WASM app model.

## Goals

- Clear separation between internal (small, critical) and external (large) storage.
- Simple, deterministic path mapping for apps and host services.
- Robust behavior when SD is missing or not mounted.
- Minimal and safe WASM-facing APIs; internal secrets stay host-only.

## Storage Model

### Internal storage (/int)

- Backed by **on-chip flash** (ESP32 NVS or a small filesystem).
- Intended for **small settings and secrets**:
  - Brightness, sound, per-app high scores.
  - WiFi networks + passwords.
  - System configuration and calibration.
- Must be available **before SD is mounted**.
- Size is limited; large files are not supported.

### External storage (/ext)

- Backed by **SD card** (FAT/exFAT).
- Intended for:
  - WASM apps and bundles.
  - App data (large files).
  - Media and user files.
- May be absent; API returns `NotMounted` when SD is missing.

## Virtual Paths and Aliases

All filesystem access uses a virtual root prefix:

- `/int/...` -> internal settings storage.
- `/ext/...` -> SD card storage.
- `/data/...` -> per-app data (alias to `/ext/apps_data/<appid>/...`).
- `/assets/...` -> per-app assets (alias to `/ext/apps_assets/<appid>/...`).

Notes:
- `/data` and `/assets` require the active app id to be known by the host.
- `/int` is not exposed directly to general apps; system-only access by default.
- A per-app internal path is reserved for small settings:
  - `/int/apps/<appid>/...`

## External Storage Layout

Recommended SD card layout:

- `/ext/apps/`          -> WASM binaries and app bundles
- `/ext/apps_data/`     -> per-app writable data
- `/ext/apps_assets/`   -> per-app read-only assets
- `/ext/media/`         -> user media
- `/ext/files/`         -> generic user files

Apps should use `/data` and `/assets` aliases rather than hard-coded paths.

## Access Policy

### System-only (host)

- WiFi credentials, secrets, and global settings live in `/int/system/...`.
- Only host-side services can read or modify those files/keys.

### App access (WASM)

- Apps may access:
  - `/data` and `/assets` (mapped to their app id on SD).
  - `/ext` (if the app opts in and the host policy allows it).
  - `/int/apps/<appid>` for small per-app settings (optional; policy-controlled).
- Apps must not access `/int/system` or other apps' internal data.

## API Design (Host / Firmware)

### Storage Manager

The firmware provides a storage service that routes by prefix and enforces policy.

```rust
enum StorageVolume { Internal, External }

enum StorageError {
    NotMounted,
    NotFound,
    NoSpace,
    PermissionDenied,
    InvalidPath,
    IoError,
}

struct StorageStat {
    is_dir: bool,
    size: u64,
}

trait StorageBackend {
    fn is_ready(&self) -> bool;
    fn open(&self, path: &str, mode: OpenMode) -> Result<FileHandle, StorageError>;
    fn read(&self, handle: &mut FileHandle, buf: &mut [u8]) -> Result<usize, StorageError>;
    fn write(&self, handle: &mut FileHandle, data: &[u8]) -> Result<usize, StorageError>;
    fn close(&self, handle: FileHandle) -> Result<(), StorageError>;
    fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, StorageError>;
    fn stat(&self, path: &str) -> Result<StorageStat, StorageError>;
    fn mkdir(&self, path: &str) -> Result<(), StorageError>;
    fn delete(&self, path: &str) -> Result<(), StorageError>;
}
```

### Path Resolution

```
/int/...     -> Internal backend
/ext/...     -> External backend
/data/...    -> /ext/apps_data/<appid>/...
/assets/...  -> /ext/apps_assets/<appid>/...
```

`<appid>` is the active app id from the app manager registry.

### Settings Store

For small settings, provide a typed key/value store on top of `/int`:

```rust
struct SettingsStore;
impl SettingsStore {
    fn get_u32(&self, namespace: &str, key: &str) -> Option<u32>;
    fn set_u32(&self, namespace: &str, key: &str, value: u32) -> Result<(), StorageError>;
    fn get_blob(&self, namespace: &str, key: &str, out: &mut [u8]) -> Result<usize, StorageError>;
    fn set_blob(&self, namespace: &str, key: &str, data: &[u8]) -> Result<(), StorageError>;
}
```

Namespaces:
- `system` (host-only): WiFi, global settings.
- `app:<appid>` (host and/or app-controlled): highscores, per-app settings.

## API Design (WASM Imports)

WASM apps use a minimal file API and a small settings API.
All paths are virtual and passed as UTF-8 C strings.

### File API (optional for apps)

```
storage_open(path, flags) -> handle
storage_read(handle, ptr, len) -> bytes_read
storage_write(handle, ptr, len) -> bytes_written
storage_close(handle)
storage_list(path, out_ptr, out_len) -> bytes_written
storage_stat(path, out_ptr, out_len) -> status
storage_mkdir(path) -> status
storage_delete(path) -> status
storage_is_ready(volume) -> bool
```

### Settings API (preferred for small data)

```
settings_get_u32(namespace, key, default) -> u32
settings_set_u32(namespace, key, value) -> status
settings_get_blob(namespace, key, out_ptr, out_len) -> bytes_written
settings_set_blob(namespace, key, ptr, len) -> status
```

Notes:
- `namespace` is limited for apps (e.g., `app:<appid>` only).
- WiFi credentials and system settings are not exposed to WASM apps.

## Behavior Requirements

- Internal storage must be available at boot (no SD dependency).
- External storage may be missing; calls return `NotMounted`.
- `/data` and `/assets` resolve to SD. If SD is absent, these paths fail with `NotMounted`.
- All path inputs are validated (no `..`, no empty segments).
- Host is responsible for enforcing app isolation.

## Suggested ESP32 Implementation

### Internal settings storage

- Use ESP32 **NVS** for key/value data.
- Optionally store small blobs for WiFi profiles and highscores.

### External SD storage

- Use FAT/exFAT (via ESP-IDF SDMMC or SPI SD).
- Mount at a fixed point and map to `/ext`.

## Testing

- Unit tests for path aliasing and access policy.
- Integration tests for:
  - SD absent -> `/ext` errors.
  - `/data` resolves to the correct app id.
  - Settings get/set round-trips.
