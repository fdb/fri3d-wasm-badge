// Minimal wasm3 host harness: loads firmware/build/*_opt.wasm, links the
// same 24 host functions as wasm_host.cpp (stubbed), and calls render() once.
// Runs on macOS/Linux so we can reproduce any wasm3+module issue without
// flashing the badge.
//
// Build:
//   cc -I ../../lib/wasm3/src \
//      ../../lib/wasm3/src/*.c hosttest.c -o hosttest
// Run:
//   ./hosttest ../../build/fri3d_app_circles_opt.wasm

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#include "wasm3.h"
#include "m3_env.h"

static const char* load_file(const char* path, uint8_t** out_buf, size_t* out_len) {
    FILE* f = fopen(path, "rb");
    if (!f) return "fopen failed";
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    uint8_t* buf = malloc(n);
    if (!buf) { fclose(f); return "oom"; }
    if (fread(buf, 1, n, f) != (size_t)n) { free(buf); fclose(f); return "short read"; }
    fclose(f);
    *out_buf = buf;
    *out_len = (size_t)n;
    return NULL;
}

// Each host fn prints its name so we can trace what the module calls.

m3ApiRawFunction(h_canvas_clear)       { printf("host: canvas_clear\n"); m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_width)       { m3ApiReturnType(int32_t); m3ApiReturn(128); }
m3ApiRawFunction(h_canvas_height)      { m3ApiReturnType(int32_t); m3ApiReturn(64); }
m3ApiRawFunction(h_canvas_set_color)   { m3ApiGetArg(int32_t, c); printf("host: canvas_set_color(%d)\n", c); m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_set_font)    { m3ApiGetArg(int32_t, f); (void)f; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_dot)    { m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); (void)x; (void)y; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_line)   { m3ApiGetArg(int32_t, x1); m3ApiGetArg(int32_t, y1); m3ApiGetArg(int32_t, x2); m3ApiGetArg(int32_t, y2); (void)x1;(void)y1;(void)x2;(void)y2; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_frame)  { m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); m3ApiGetArg(int32_t, w); m3ApiGetArg(int32_t, h); (void)x;(void)y;(void)w;(void)h; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_box)    { m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); m3ApiGetArg(int32_t, w); m3ApiGetArg(int32_t, h); (void)x;(void)y;(void)w;(void)h; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_rframe) { m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); m3ApiGetArg(int32_t, w); m3ApiGetArg(int32_t, h); m3ApiGetArg(int32_t, r); (void)x;(void)y;(void)w;(void)h;(void)r; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_rbox)   { m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); m3ApiGetArg(int32_t, w); m3ApiGetArg(int32_t, h); m3ApiGetArg(int32_t, r); (void)x;(void)y;(void)w;(void)h;(void)r; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_circle) {
    m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); m3ApiGetArg(int32_t, r);
    printf("host: canvas_draw_circle(%d, %d, %d)\n", x, y, r);
    m3ApiSuccess();
}
m3ApiRawFunction(h_canvas_draw_disc)   { m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); m3ApiGetArg(int32_t, r); (void)x;(void)y;(void)r; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_draw_str)    { m3ApiGetArg(int32_t, x); m3ApiGetArg(int32_t, y); m3ApiGetArgMem(const char*, t); (void)x;(void)y;(void)t; m3ApiSuccess(); }
m3ApiRawFunction(h_canvas_string_width){ m3ApiReturnType(int32_t); m3ApiGetArgMem(const char*, t); m3ApiReturn(t ? (int32_t)strlen(t) * 6 : 0); }

static uint32_t g_seed = 42;
// Minimal xorshift; the desktop harness doesn't need to match MT19937 bit-exactly.
static uint32_t fake_rand() { g_seed ^= g_seed << 13; g_seed ^= g_seed >> 17; g_seed ^= g_seed << 5; return g_seed; }

m3ApiRawFunction(h_random_seed)        { m3ApiGetArg(int32_t, s); g_seed = (uint32_t)s ? (uint32_t)s : 1; printf("host: random_seed(%d)\n", s); m3ApiSuccess(); }
m3ApiRawFunction(h_random_get)         { m3ApiReturnType(int32_t); m3ApiReturn((int32_t)fake_rand()); }
m3ApiRawFunction(h_random_range)       { m3ApiReturnType(int32_t); m3ApiGetArg(int32_t, max); int32_t v = max > 0 ? (int32_t)(fake_rand() % (uint32_t)max) : 0; printf("host: random_range(%d) -> %d\n", max, v); m3ApiReturn(v); }
m3ApiRawFunction(h_get_time_ms)        { m3ApiReturnType(int32_t); m3ApiReturn(0); }
m3ApiRawFunction(h_start_timer_ms)     { m3ApiGetArg(int32_t, i); (void)i; m3ApiSuccess(); }
m3ApiRawFunction(h_stop_timer)         { m3ApiSuccess(); }
m3ApiRawFunction(h_request_render)     { m3ApiSuccess(); }
m3ApiRawFunction(h_exit_to_launcher)   { printf("host: exit_to_launcher\n"); m3ApiSuccess(); }
m3ApiRawFunction(h_start_app)          { m3ApiGetArg(int32_t, id); (void)id; m3ApiSuccess(); }

static M3Result link_all(IM3Module mod) {
#define LINK(sig, name) do { \
        M3Result r = m3_LinkRawFunction(mod, "env", #name, sig, &h_##name); \
        if (r && r != m3Err_functionLookupFailed) return r; \
    } while (0)

    LINK("v()",       canvas_clear);
    LINK("i()",       canvas_width);
    LINK("i()",       canvas_height);
    LINK("v(i)",      canvas_set_color);
    LINK("v(i)",      canvas_set_font);
    LINK("v(ii)",     canvas_draw_dot);
    LINK("v(iiii)",   canvas_draw_line);
    LINK("v(iiii)",   canvas_draw_frame);
    LINK("v(iiii)",   canvas_draw_box);
    LINK("v(iiiii)",  canvas_draw_rframe);
    LINK("v(iiiii)",  canvas_draw_rbox);
    LINK("v(iii)",    canvas_draw_circle);
    LINK("v(iii)",    canvas_draw_disc);
    LINK("v(iii)",    canvas_draw_str);
    LINK("i(i)",      canvas_string_width);
    LINK("v(i)",      random_seed);
    LINK("i()",       random_get);
    LINK("i(i)",      random_range);
    LINK("i()",       get_time_ms);
    LINK("v(i)",      start_timer_ms);
    LINK("v()",       stop_timer);
    LINK("v()",       request_render);
    LINK("v()",       exit_to_launcher);
    LINK("v(i)",      start_app);
#undef LINK
    return NULL;
}

int main(int argc, char** argv) {
    if (argc < 2) { fprintf(stderr, "usage: %s <wasm>\n", argv[0]); return 1; }

    uint8_t* wasm_buf = NULL; size_t wasm_len = 0;
    const char* lerr = load_file(argv[1], &wasm_buf, &wasm_len);
    if (lerr) { fprintf(stderr, "load: %s\n", lerr); return 1; }
    printf("loaded %zu bytes\n", wasm_len);

    IM3Environment env = m3_NewEnvironment();
    IM3Runtime rt = m3_NewRuntime(env, 8192, NULL);
    IM3Module mod = NULL;

    M3Result r = m3_ParseModule(env, &mod, wasm_buf, (uint32_t)wasm_len);
    if (r) { fprintf(stderr, "parse: %s\n", r); return 1; }
    r = m3_LoadModule(rt, mod);
    if (r) { fprintf(stderr, "load: %s\n", r); return 1; }

    r = link_all(mod);
    if (r) { fprintf(stderr, "link: %s\n", r); return 1; }

    IM3Function fn = NULL;
    r = m3_FindFunction(&fn, rt, "render");
    if (r) { fprintf(stderr, "find render: %s\n", r); return 1; }
    printf("calling render...\n");
    r = m3_CallV(fn);
    if (r) { fprintf(stderr, "render trap: %s\n", r); return 1; }
    printf("render ok\n");

    // Try on_input too, with KEY_OK PRESS.
    IM3Function oi = NULL;
    r = m3_FindFunction(&oi, rt, "on_input");
    if (!r) {
        printf("calling on_input(OK=4, PRESS=0)...\n");
        r = m3_CallV(oi, (int32_t)4, (int32_t)0);
        if (r) { fprintf(stderr, "on_input trap: %s\n", r); return 1; }
        printf("on_input ok\n");
    }

    return 0;
}
