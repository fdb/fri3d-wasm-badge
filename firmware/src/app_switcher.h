#pragma once

// Resolves pending start_app / exit_to_launcher requests the running WASM
// module has made, by looking up the target in EMBEDDED_APPS and asking
// wasm_host to reload it. Shared by firmware/src/main.cpp (hardware) and
// firmware/web/main_web.cpp (browser) so the dispatch logic can't drift.

namespace app_switcher {

// Call after each wasm_host::render() / wasm_host::on_input(). A no-op if
// neither exit_requested() nor start_app_requested() is set.
void dispatch();

} // namespace app_switcher
