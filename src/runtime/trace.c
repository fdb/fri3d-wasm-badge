#include "trace.h"

#ifdef FRD_TRACE

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TRACE_MAX_ARGS 8
#define TRACE_INITIAL_CAPACITY 256

typedef struct {
    uint32_t frame;
    const char* fn;
    size_t argc;
    trace_arg_t args[TRACE_MAX_ARGS];
    bool has_ret;
    int64_t ret;
} trace_event_t;

typedef struct {
    trace_event_t* events;
    size_t count;
    size_t capacity;
    uint32_t current_frame;
} trace_state_t;

static trace_state_t g_trace = {0};

static char* trace_strdup(const char* str) {
    if (!str) {
        return NULL;
    }
    size_t len = strlen(str);
    char* copy = (char*)malloc(len + 1);
    if (!copy) {
        return NULL;
    }
    memcpy(copy, str, len);
    copy[len] = '\0';
    return copy;
}

static void trace_event_clear(trace_event_t* event) {
    if (!event) {
        return;
    }
    for (size_t i = 0; i < event->argc; i++) {
        if (event->args[i].type == trace_arg_str && event->args[i].value.str) {
            free((void*)event->args[i].value.str);
            event->args[i].value.str = NULL;
        }
    }
}

static bool trace_ensure_capacity(size_t needed) {
    if (g_trace.capacity >= needed) {
        return true;
    }

    size_t new_capacity = g_trace.capacity ? g_trace.capacity : TRACE_INITIAL_CAPACITY;
    while (new_capacity < needed) {
        new_capacity *= 2;
    }

    trace_event_t* new_events = (trace_event_t*)realloc(g_trace.events, new_capacity * sizeof(trace_event_t));
    if (!new_events) {
        return false;
    }
    g_trace.events = new_events;
    g_trace.capacity = new_capacity;
    return true;
}

void trace_reset(void) {
    for (size_t i = 0; i < g_trace.count; i++) {
        trace_event_clear(&g_trace.events[i]);
    }
    g_trace.count = 0;
    g_trace.current_frame = 0;
}

void trace_begin(uint32_t frame) {
    g_trace.current_frame = frame;
}

void trace_call(const char* fn, const trace_arg_t* args, size_t argc) {
    if (!trace_ensure_capacity(g_trace.count + 1)) {
        return;
    }

    trace_event_t* event = &g_trace.events[g_trace.count++];
    memset(event, 0, sizeof(*event));
    event->frame = g_trace.current_frame;
    event->fn = fn ? fn : "";

    if (argc > TRACE_MAX_ARGS) {
        argc = TRACE_MAX_ARGS;
    }
    event->argc = argc;

    for (size_t i = 0; i < argc; i++) {
        event->args[i].type = args[i].type;
        if (args[i].type == trace_arg_str) {
            event->args[i].value.str = trace_strdup(args[i].value.str ? args[i].value.str : "");
        } else {
            event->args[i].value.i64 = args[i].value.i64;
        }
    }
}

void trace_result(int64_t value) {
    if (g_trace.count == 0) {
        return;
    }
    trace_event_t* event = &g_trace.events[g_trace.count - 1];
    event->has_ret = true;
    event->ret = value;
}

static void trace_write_json_string(FILE* file, const char* str) {
    fputc('"', file);
    if (str) {
        for (const char* p = str; *p; p++) {
            unsigned char c = (unsigned char)*p;
            switch (c) {
                case '\\':
                    fputs("\\\\", file);
                    break;
                case '"':
                    fputs("\\\"", file);
                    break;
                case '\n':
                    fputs("\\n", file);
                    break;
                case '\r':
                    fputs("\\r", file);
                    break;
                case '\t':
                    fputs("\\t", file);
                    break;
                default:
                    if (c < 0x20) {
                        fprintf(file, "\\u%04x", c);
                    } else {
                        fputc(c, file);
                    }
                    break;
            }
        }
    }
    fputc('"', file);
}

bool trace_write_json(const char* path, const char* app, uint32_t seed, uint32_t frames) {
    if (!path) {
        return false;
    }

    FILE* file = fopen(path, "w");
    if (!file) {
        return false;
    }

    fprintf(file, "{\n");
    fprintf(file, "  \"app\": ");
    trace_write_json_string(file, app ? app : "");
    fprintf(file, ",\n  \"seed\": %u,\n  \"frames\": %u,\n  \"events\": [\n",
            seed, frames);

    for (size_t i = 0; i < g_trace.count; i++) {
        trace_event_t* event = &g_trace.events[i];
        fprintf(file, "    {\"frame\": %u, \"fn\": ", event->frame);
        trace_write_json_string(file, event->fn);
        fprintf(file, ", \"args\": [");

        for (size_t a = 0; a < event->argc; a++) {
            if (a > 0) {
                fputs(", ", file);
            }
            if (event->args[a].type == trace_arg_str) {
                trace_write_json_string(file, event->args[a].value.str);
            } else {
                fprintf(file, "%" PRId64, event->args[a].value.i64);
            }
        }

        fprintf(file, "]");

        if (event->has_ret) {
            fprintf(file, ", \"ret\": %" PRId64, event->ret);
        }

        fprintf(file, "}%s\n", (i + 1 < g_trace.count) ? "," : "");
    }

    fprintf(file, "  ]\n}\n");
    fclose(file);
    return true;
}

#endif
