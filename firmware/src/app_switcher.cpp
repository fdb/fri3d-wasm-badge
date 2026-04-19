#include "app_switcher.h"

#include "embedded_apps.h"
#include "host_platform.h"
#include "wasm_host.h"

namespace app_switcher {

void dispatch() {
    if (wasm_host::exit_requested()) {
        wasm_host::clear_exit_request();
        const EmbeddedApp& launcher = EMBEDDED_APPS[0];
        host_log("[fri3d] exit_to_launcher -> %s\n", launcher.name);
        const char* err = wasm_host::reload(launcher.wasm, launcher.wasm_len);
        if (err) host_log("[fri3d] launcher reload failed: %s\n", err);
    }

    if (wasm_host::start_app_requested()) {
        uint32_t req = wasm_host::requested_app_id();
        wasm_host::clear_start_app_request();
        const EmbeddedApp* target = nullptr;
        for (uint32_t i = 0; i < EMBEDDED_APPS_COUNT; ++i) {
            if (EMBEDDED_APPS[i].app_id == req) { target = &EMBEDDED_APPS[i]; break; }
        }
        if (!target) {
            host_log("[fri3d] start_app(%u): no such app\n", req);
            return;
        }
        host_log("[fri3d] start_app(%u) -> %s\n", req, target->name);
        const char* err = wasm_host::reload(target->wasm, target->wasm_len);
        if (err) host_log("[fri3d] %s reload failed: %s\n", target->name, err);
    }
}

} // namespace app_switcher
