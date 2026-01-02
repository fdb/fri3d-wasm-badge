#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef enum {
    trace_arg_int = 0,
    trace_arg_str = 1
} trace_arg_type_t;

typedef struct {
    trace_arg_type_t type;
    union {
        int64_t i64;
        const char* str;
    } value;
} trace_arg_t;

#define TRACE_ARG_INT(v) ((trace_arg_t){ .type = trace_arg_int, .value.i64 = (int64_t)(v) })
#define TRACE_ARG_STR(v) ((trace_arg_t){ .type = trace_arg_str, .value.str = (v) ? (v) : "" })

#ifdef FRD_TRACE
void trace_reset(void);
void trace_begin(uint32_t frame);
void trace_call(const char* fn, const trace_arg_t* args, size_t argc);
void trace_result(int64_t value);
bool trace_write_json(const char* path, const char* app, uint32_t seed, uint32_t frames);
#else
static inline void trace_reset(void) { }
static inline void trace_begin(uint32_t frame) { (void)frame; }
static inline void trace_call(const char* fn, const trace_arg_t* args, size_t argc) {
    (void)fn;
    (void)args;
    (void)argc;
}
static inline void trace_result(int64_t value) { (void)value; }
static inline bool trace_write_json(const char* path, const char* app, uint32_t seed, uint32_t frames) {
    (void)path;
    (void)app;
    (void)seed;
    (void)frames;
    return true;
}
#endif
