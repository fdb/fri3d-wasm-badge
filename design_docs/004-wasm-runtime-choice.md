# 004 — WASM runtime, HAL and AOT options

## Runtime: wasmi 1.1

| Option | Verdict |
| --- | --- |
| **wasmi 1.1** (pure Rust) | **Chosen.** `no_std` with `default-features = false`, register-based IR, fuel metering, resumable calls, `StoreLimits`. Two deps (`spin`, `wasmparser`). Audited twice. Runs inside the browser when compiled to wasm32. |
| wasm3-rs | Dead since 2022, C + bindgen, soundness caveats. |
| WAMR | C. Smallest flash, has Xtensa AOT via `wamrc`. Not Rust. |
| Wasmtime / Cranelift | No 32-bit, no Xtensa backend. Pulley (portable bytecode) is `no_std` but ~10× slower than native and still an interpreter. |

Decisions inside wasmi:
- `consume_fuel(true)`; refill per call. Costs ~10 % interpretation
  speed; buys a kernel that survives any app.
- `prefer-btree-collections`: wasm32 and bare metal have no entropy for
  hash maps.
- Eager compilation: pay translation at app start, never mid-frame.
- 2.0 is in beta (faster execution, threaded dispatch). Upgrade when it
  ships; the host-import code is API-stable across 1.x → 2.x.

## HAL: esp-hal 1.1 (no_std)

esp-hal is where Espressif invests; builds are pure Rust (no ESP-IDF C
tree, no `ldproxy`). `esp-alloc` places the heap in PSRAM
(`psram_allocator!`) so wasmi's linear memories live there while the
interpreter stack stays in internal SRAM. `mipidsi 0.10` drives the
ST7789V over `embedded-hal` SPI. `esp-radio` adds Wi-Fi later.

esp-idf-hal (std) remains the fallback if we ever need mbedTLS or a
C component. Nothing in the kernel depends on `std`.

## AOT / native compile service

There is no Rust-native AOT for Xtensa. The options, in order of effort:

1. **Interpreter on badge (now).** wasmi, fuel-capped. Good enough for
   UI apps; ~100–300 ms per frame for Mandelbrot-class compute.
2. **Build service → native ELF.** `wasm2c`/`w2c2` emit C; the Xtensa
   GCC in `~/.espressif/tools/xtensa-esp-elf` compiles it; the badge
   loads a relocatable blob. Needs a loader and a symbol table for the
   host imports. 10–30× faster. See `specs/014-aot-compilation.md`.
3. **WAMR AOT.** Proven on ESP32 but replaces the Rust runtime with C.

The `.fab` bundle format reserves header bytes for a `payload_kind`
field so a future native payload slots in beside `.wasm` without a new
format. The build service is the same pipeline as `fri3d-pack`, plus
one step; an app store would host that service and sign the output.

## Browser host

The same kernel crate compiles to `wasm32-unknown-unknown` with
`wasm-bindgen`. wasmi-in-wasm works; the guest memory lives inside the
host module's memory. Avoid `getrandom` in the dependency tree.
