#include "canvas.h"
#include "input.h"
#include "random.h"
#include "trace.h"
#include "wasm_runner.h"

#include "jsmn.h"

#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <u8g2.h>

#define TRACE_HEAP_SIZE (10 * 1024 * 1024)
#define DEFAULT_FRAME_MS 16
#define DEFAULT_SHORT_PRESS_MS 10
typedef enum {
    script_event_press = 0,
    script_event_release,
    script_event_short_press,
    script_event_long_press,
    script_event_repeat
} script_event_type_t;

typedef struct {
    uint32_t time_ms;
    input_key_t key;
    script_event_type_t type;
    uint32_t duration_ms;
    bool has_duration;
} script_event_t;

typedef struct {
    uint32_t time_ms;
    input_key_t key;
    input_type_t type;
} raw_event_t;

typedef struct {
    uint32_t time_ms;
    input_key_t key;
    input_type_t type;
} timed_event_t;

typedef struct {
    const char* wasm_path;
    const char* output_path;
    const char* input_path;
    const char* app_id;
    uint32_t frames;
    uint32_t seed;
    uint32_t frame_ms;
    int scene;
} options_t;

static void print_usage(const char* program) {
    fprintf(stderr,
            "Usage: %s --app <path> --out <trace.json> [options]\n\n"
            "Options:\n"
            "  --frames <n>     Number of render frames (default: 1)\n"
            "  --seed <n>       RNG seed (default: 42)\n"
            "  --frame-ms <n>   Frame duration in ms (default: 16)\n"
            "  --scene <n>      Set scene (if supported by app)\n"
            "  --input <path>   Input script JSON\n"
            "  --app-id <id>    App id for trace metadata\n"
            "  --help           Show this help\n",
            program);
}

static bool parse_args(int argc, char* argv[], options_t* options) {
    if (!options) {
        return false;
    }

    memset(options, 0, sizeof(*options));
    options->frames = 1;
    options->seed = 42;
    options->frame_ms = DEFAULT_FRAME_MS;
    options->scene = -1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--app") == 0 && i + 1 < argc) {
            options->wasm_path = argv[++i];
        } else if (strcmp(argv[i], "--out") == 0 && i + 1 < argc) {
            options->output_path = argv[++i];
        } else if (strcmp(argv[i], "--input") == 0 && i + 1 < argc) {
            options->input_path = argv[++i];
        } else if (strcmp(argv[i], "--frames") == 0 && i + 1 < argc) {
            options->frames = (uint32_t)strtoul(argv[++i], NULL, 10);
        } else if (strcmp(argv[i], "--seed") == 0 && i + 1 < argc) {
            options->seed = (uint32_t)strtoul(argv[++i], NULL, 10);
        } else if (strcmp(argv[i], "--frame-ms") == 0 && i + 1 < argc) {
            options->frame_ms = (uint32_t)strtoul(argv[++i], NULL, 10);
        } else if (strcmp(argv[i], "--scene") == 0 && i + 1 < argc) {
            options->scene = atoi(argv[++i]);
        } else if (strcmp(argv[i], "--app-id") == 0 && i + 1 < argc) {
            options->app_id = argv[++i];
        } else if (strcmp(argv[i], "--help") == 0) {
            print_usage(argv[0]);
            return false;
        } else {
            fprintf(stderr, "Unknown option: %s\n", argv[i]);
            print_usage(argv[0]);
            return false;
        }
    }

    if (!options->wasm_path || !options->output_path) {
        fprintf(stderr, "Error: --app and --out are required\n");
        print_usage(argv[0]);
        return false;
    }

    if (options->frame_ms == 0) {
        options->frame_ms = DEFAULT_FRAME_MS;
    }

    return true;
}

static void trace_exit_callback(void* context) {
    (void)context;
}

static void trace_start_callback(uint32_t app_id, void* context) {
    (void)app_id;
    (void)context;
}

static bool json_token_eq(const char* json, const jsmntok_t* tok, const char* str) {
    size_t len = (size_t)(tok->end - tok->start);
    return tok->type == JSMN_STRING &&
           strlen(str) == len &&
           strncmp(json + tok->start, str, len) == 0;
}

static bool json_token_to_uint32(const char* json, const jsmntok_t* tok, uint32_t* out) {
    if (!out || tok->type != JSMN_PRIMITIVE) {
        return false;
    }
    size_t len = (size_t)(tok->end - tok->start);
    if (len == 0 || len >= 32) {
        return false;
    }
    char buf[32];
    memcpy(buf, json + tok->start, len);
    buf[len] = '\0';
    char* end = NULL;
    unsigned long value = strtoul(buf, &end, 10);
    if (!end || *end != '\0') {
        return false;
    }
    *out = (uint32_t)value;
    return true;
}

static bool parse_key(const char* json, const jsmntok_t* tok, input_key_t* out) {
    if (!out || tok->type != JSMN_STRING) {
        return false;
    }
    if (json_token_eq(json, tok, "up")) {
        *out = input_key_up;
    } else if (json_token_eq(json, tok, "down")) {
        *out = input_key_down;
    } else if (json_token_eq(json, tok, "left")) {
        *out = input_key_left;
    } else if (json_token_eq(json, tok, "right")) {
        *out = input_key_right;
    } else if (json_token_eq(json, tok, "ok")) {
        *out = input_key_ok;
    } else if (json_token_eq(json, tok, "back")) {
        *out = input_key_back;
    } else {
        return false;
    }
    return true;
}

static bool parse_type(const char* json, const jsmntok_t* tok, script_event_type_t* out) {
    if (!out || tok->type != JSMN_STRING) {
        return false;
    }
    if (json_token_eq(json, tok, "press")) {
        *out = script_event_press;
    } else if (json_token_eq(json, tok, "release")) {
        *out = script_event_release;
    } else if (json_token_eq(json, tok, "short_press")) {
        *out = script_event_short_press;
    } else if (json_token_eq(json, tok, "long_press")) {
        *out = script_event_long_press;
    } else if (json_token_eq(json, tok, "repeat")) {
        *out = script_event_repeat;
    } else {
        return false;
    }
    return true;
}

static bool append_raw_event(raw_event_t** events, size_t* count, size_t* capacity,
                             uint32_t time_ms, input_key_t key, input_type_t type) {
    if (!events || !count || !capacity) {
        return false;
    }
    if (*count + 1 > *capacity) {
        size_t new_capacity = (*capacity == 0) ? 32 : (*capacity * 2);
        raw_event_t* new_events = (raw_event_t*)realloc(*events, new_capacity * sizeof(raw_event_t));
        if (!new_events) {
            return false;
        }
        *events = new_events;
        *capacity = new_capacity;
    }
    (*events)[*count].time_ms = time_ms;
    (*events)[*count].key = key;
    (*events)[*count].type = type;
    (*count)++;
    return true;
}

static int raw_event_compare(const void* a, const void* b) {
    const raw_event_t* lhs = (const raw_event_t*)a;
    const raw_event_t* rhs = (const raw_event_t*)b;
    if (lhs->time_ms < rhs->time_ms) {
        return -1;
    }
    if (lhs->time_ms > rhs->time_ms) {
        return 1;
    }
    if (lhs->type != rhs->type) {
        return (lhs->type == input_type_press) ? -1 : 1;
    }
    if (lhs->key < rhs->key) {
        return -1;
    }
    if (lhs->key > rhs->key) {
        return 1;
    }
    return 0;
}

static bool expand_script_events(const script_event_t* script, size_t script_count,
                                 raw_event_t** out_events, size_t* out_count) {
    size_t capacity = 0;
    size_t count = 0;
    raw_event_t* events = NULL;

    for (size_t i = 0; i < script_count; i++) {
        const script_event_t* event = &script[i];
        uint32_t duration = event->has_duration ? event->duration_ms : 0;
        switch (event->type) {
            case script_event_press:
                if (!append_raw_event(&events, &count, &capacity, event->time_ms, event->key, input_type_press)) {
                    free(events);
                    return false;
                }
                break;
            case script_event_release:
                if (!append_raw_event(&events, &count, &capacity, event->time_ms, event->key, input_type_release)) {
                    free(events);
                    return false;
                }
                break;
            case script_event_short_press:
                if (duration == 0) {
                    duration = DEFAULT_SHORT_PRESS_MS;
                }
                if (!append_raw_event(&events, &count, &capacity, event->time_ms, event->key, input_type_press) ||
                    !append_raw_event(&events, &count, &capacity, event->time_ms + duration,
                                      event->key, input_type_release)) {
                    free(events);
                    return false;
                }
                break;
            case script_event_long_press: {
                if (duration < INPUT_LONG_PRESS_MS) {
                    duration = INPUT_LONG_PRESS_MS;
                }
                if (!append_raw_event(&events, &count, &capacity, event->time_ms, event->key, input_type_press) ||
                    !append_raw_event(&events, &count, &capacity, event->time_ms + duration,
                                      event->key, input_type_release)) {
                    free(events);
                    return false;
                }
                break;
            }
            case script_event_repeat: {
                uint32_t min_duration = INPUT_REPEAT_START_MS + INPUT_REPEAT_INTERVAL_MS;
                if (duration < min_duration) {
                    duration = min_duration;
                }
                if (!append_raw_event(&events, &count, &capacity, event->time_ms, event->key, input_type_press) ||
                    !append_raw_event(&events, &count, &capacity, event->time_ms + duration,
                                      event->key, input_type_release)) {
                    free(events);
                    return false;
                }
                break;
            }
            default:
                break;
        }
    }

    if (count > 1) {
        qsort(events, count, sizeof(raw_event_t), raw_event_compare);
    }

    *out_events = events;
    *out_count = count;
    return true;
}

static bool append_timed_event(timed_event_t** events, size_t* count, size_t* capacity,
                               uint32_t time_ms, input_key_t key, input_type_t type) {
    if (!events || !count || !capacity) {
        return false;
    }
    if (*count + 1 > *capacity) {
        size_t new_capacity = (*capacity == 0) ? 32 : (*capacity * 2);
        timed_event_t* new_events = (timed_event_t*)realloc(*events, new_capacity * sizeof(timed_event_t));
        if (!new_events) {
            return false;
        }
        *events = new_events;
        *capacity = new_capacity;
    }
    (*events)[*count].time_ms = time_ms;
    (*events)[*count].key = key;
    (*events)[*count].type = type;
    (*count)++;
    return true;
}

static int timed_event_weight(input_type_t type) {
    switch (type) {
        case input_type_press:
            return 0;
        case input_type_long_press:
        case input_type_short_press:
        case input_type_repeat:
            return 1;
        case input_type_release:
            return 2;
        default:
            return 3;
    }
}

static int timed_event_compare(const void* a, const void* b) {
    const timed_event_t* lhs = (const timed_event_t*)a;
    const timed_event_t* rhs = (const timed_event_t*)b;
    if (lhs->time_ms < rhs->time_ms) {
        return -1;
    }
    if (lhs->time_ms > rhs->time_ms) {
        return 1;
    }
    int lw = timed_event_weight(lhs->type);
    int rw = timed_event_weight(rhs->type);
    if (lw != rw) {
        return (lw < rw) ? -1 : 1;
    }
    if (lhs->key < rhs->key) {
        return -1;
    }
    if (lhs->key > rhs->key) {
        return 1;
    }
    return 0;
}

static bool build_timed_events(const raw_event_t* raw_events, size_t raw_count,
                               timed_event_t** out_events, size_t* out_count) {
    if (!out_events || !out_count) {
        return false;
    }

    timed_event_t* events = NULL;
    size_t count = 0;
    size_t capacity = 0;

    uint32_t press_times[INPUT_KEY_COUNT] = {0};
    bool pressed[INPUT_KEY_COUNT] = {0};

    for (size_t i = 0; i < raw_count; i++) {
        const raw_event_t* raw = &raw_events[i];
        if (raw->key >= INPUT_KEY_COUNT) {
            continue;
        }

        if (!append_timed_event(&events, &count, &capacity, raw->time_ms, raw->key, raw->type)) {
            free(events);
            return false;
        }

        if (raw->type == input_type_press) {
            pressed[raw->key] = true;
            press_times[raw->key] = raw->time_ms;
        } else if (raw->type == input_type_release) {
            if (!pressed[raw->key]) {
                continue;
            }
            pressed[raw->key] = false;
            uint32_t press_time = press_times[raw->key];
            uint32_t release_time = raw->time_ms;
            uint32_t hold_time = release_time - press_time;

            if (hold_time >= INPUT_LONG_PRESS_MS) {
                uint32_t long_press_time = press_time + INPUT_LONG_PRESS_MS;
                if (!append_timed_event(&events, &count, &capacity,
                                        long_press_time, raw->key, input_type_long_press)) {
                    free(events);
                    return false;
                }

                uint32_t repeat_time = long_press_time + INPUT_REPEAT_INTERVAL_MS;
                while (repeat_time < release_time) {
                    if (!append_timed_event(&events, &count, &capacity,
                                            repeat_time, raw->key, input_type_repeat)) {
                        free(events);
                        return false;
                    }
                    repeat_time += INPUT_REPEAT_INTERVAL_MS;
                }
            } else {
                if (!append_timed_event(&events, &count, &capacity,
                                        release_time, raw->key, input_type_short_press)) {
                    free(events);
                    return false;
                }
            }
        }
    }

    if (count > 1) {
        qsort(events, count, sizeof(timed_event_t), timed_event_compare);
    }

    *out_events = events;
    *out_count = count;
    return true;
}

static bool load_input_script(const char* path, raw_event_t** out_events, size_t* out_count) {
    *out_events = NULL;
    *out_count = 0;
    if (!path) {
        return true;
    }

    FILE* file = fopen(path, "rb");
    if (!file) {
        fprintf(stderr, "Failed to open input script: %s\n", path);
        return false;
    }

    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    rewind(file);

    if (size <= 0) {
        fclose(file);
        return true;
    }

    char* json = (char*)malloc((size_t)size + 1);
    if (!json) {
        fclose(file);
        return false;
    }

    if (fread(json, 1, (size_t)size, file) != (size_t)size) {
        free(json);
        fclose(file);
        return false;
    }
    json[size] = '\0';
    fclose(file);

    size_t token_capacity = 256 + (size_t)size / 8;
    jsmntok_t* tokens = (jsmntok_t*)calloc(token_capacity, sizeof(jsmntok_t));
    if (!tokens) {
        free(json);
        return false;
    }

    jsmn_parser parser;
    jsmn_init(&parser);
    int token_count = jsmn_parse(&parser, json, (size_t)size, tokens, (unsigned int)token_capacity);
    if (token_count < 1 || tokens[0].type != JSMN_ARRAY) {
        fprintf(stderr, "Input script must be a JSON array\n");
        free(tokens);
        free(json);
        return false;
    }

    size_t script_capacity = (size_t)tokens[0].size;
    script_event_t* script_events = (script_event_t*)calloc(script_capacity, sizeof(script_event_t));
    if (!script_events && script_capacity > 0) {
        free(tokens);
        free(json);
        return false;
    }

    int idx = 1;
    size_t script_count = 0;
    for (int i = 0; i < tokens[0].size; i++) {
        if (tokens[idx].type != JSMN_OBJECT) {
            fprintf(stderr, "Input event must be an object\n");
            free(script_events);
            free(tokens);
            free(json);
            return false;
        }
        int obj_size = tokens[idx].size;
        idx++;

        script_event_t event = {0};
        bool has_time = false;
        bool has_key = false;
        bool has_type = false;

        for (int pair = 0; pair < obj_size; pair++) {
            jsmntok_t* key_tok = &tokens[idx++];
            jsmntok_t* val_tok = &tokens[idx++];

            if (json_token_eq(json, key_tok, "time_ms")) {
                uint32_t value = 0;
                if (!json_token_to_uint32(json, val_tok, &value)) {
                    fprintf(stderr, "Invalid time_ms value\n");
                    free(script_events);
                    free(tokens);
                    free(json);
                    return false;
                }
                event.time_ms = value;
                has_time = true;
            } else if (json_token_eq(json, key_tok, "key")) {
                if (!parse_key(json, val_tok, &event.key)) {
                    fprintf(stderr, "Invalid key in input script\n");
                    free(script_events);
                    free(tokens);
                    free(json);
                    return false;
                }
                has_key = true;
            } else if (json_token_eq(json, key_tok, "type")) {
                if (!parse_type(json, val_tok, &event.type)) {
                    fprintf(stderr, "Invalid type in input script\n");
                    free(script_events);
                    free(tokens);
                    free(json);
                    return false;
                }
                has_type = true;
            } else if (json_token_eq(json, key_tok, "duration_ms")) {
                uint32_t value = 0;
                if (!json_token_to_uint32(json, val_tok, &value)) {
                    fprintf(stderr, "Invalid duration_ms value\n");
                    free(script_events);
                    free(tokens);
                    free(json);
                    return false;
                }
                event.duration_ms = value;
                event.has_duration = true;
            }
        }

        if (!has_time || !has_key || !has_type) {
            fprintf(stderr, "Input event missing required fields\n");
            free(script_events);
            free(tokens);
            free(json);
            return false;
        }

        script_events[script_count++] = event;
    }

    bool ok = expand_script_events(script_events, script_count, out_events, out_count);
    free(script_events);
    free(tokens);
    free(json);
    return ok;
}

static uint8_t trace_u8x8_gpio_and_delay(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8;
    (void)msg;
    (void)arg_int;
    (void)arg_ptr;
    return 1;
}

static uint8_t trace_u8x8_byte(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8;
    (void)msg;
    (void)arg_int;
    (void)arg_ptr;
    return 1;
}

static void setup_u8g2(u8g2_t* u8g2) {
    u8g2_Setup_ssd1306_128x64_noname_f(
        u8g2, U8G2_R0, trace_u8x8_byte, trace_u8x8_gpio_and_delay
    );
    u8g2_InitDisplay(u8g2);
    u8g2_SetPowerSave(u8g2, 0);
    u8g2_ClearBuffer(u8g2);
}

int main(int argc, char* argv[]) {
    options_t options;
    if (!parse_args(argc, argv, &options)) {
        return (argc > 1 && strcmp(argv[1], "--help") == 0) ? 0 : 1;
    }

    raw_event_t* raw_events = NULL;
    size_t raw_event_count = 0;
    if (!load_input_script(options.input_path, &raw_events, &raw_event_count)) {
        return 1;
    }

    timed_event_t* timed_events = NULL;
    size_t timed_event_count = 0;
    if (!build_timed_events(raw_events, raw_event_count, &timed_events, &timed_event_count)) {
        fprintf(stderr, "Failed to build input schedule\n");
        free(raw_events);
        return 1;
    }

    u8g2_t u8g2;
    setup_u8g2(&u8g2);

    canvas_t canvas;
    canvas_init(&canvas, &u8g2);

    random_t random;
    random_seed(&random, options.seed);

    wasm_runner_t runner;
    memset(&runner, 0, sizeof(runner));
    if (!wasm_runner_init(&runner, TRACE_HEAP_SIZE)) {
        fprintf(stderr, "Failed to init wasm runner: %s\n", wasm_runner_get_last_error(&runner));
        free(timed_events);
        free(raw_events);
        return 1;
    }

    wasm_runner_set_canvas(&runner, &canvas);
    wasm_runner_set_random(&runner, &random);
    wasm_runner_set_app_callbacks(&runner, trace_exit_callback, trace_start_callback, NULL);

    if (!wasm_runner_load_module(&runner, options.wasm_path)) {
        fprintf(stderr, "Failed to load module: %s\n", wasm_runner_get_last_error(&runner));
        wasm_runner_deinit(&runner);
        free(timed_events);
        free(raw_events);
        return 1;
    }

    if (options.scene >= 0) {
        wasm_runner_set_scene(&runner, (uint32_t)options.scene);
    }

    trace_reset();

    uint32_t frame_ms = options.frame_ms ? options.frame_ms : DEFAULT_FRAME_MS;
    uint32_t last_render_time = options.frames > 0 ? (options.frames - 1) * frame_ms : 0;
    uint32_t last_input_time = timed_event_count > 0 ? timed_events[timed_event_count - 1].time_ms : 0;
    uint32_t end_time = last_render_time;
    if (last_input_time > end_time) {
        end_time = last_input_time;
    }

    uint32_t frame_index = 0;
    uint32_t next_frame_time = options.frames > 0 ? 0 : UINT32_MAX;
    size_t next_event_index = 0;
    uint32_t next_event_time = timed_event_count > 0 ? timed_events[0].time_ms : UINT32_MAX;
    uint32_t current_time = 0;

    while (true) {
        uint32_t next_time = next_frame_time;
        if (next_event_time < next_time) {
            next_time = next_event_time;
        }

        if (next_time == UINT32_MAX) {
            break;
        }
        if (next_time > end_time) {
            break;
        }

        current_time = next_time;

        if (current_time == next_frame_time && frame_index < options.frames) {
            trace_begin(frame_index);
        }

        while (next_event_index < timed_event_count &&
               timed_events[next_event_index].time_ms == current_time) {
            timed_event_t event = timed_events[next_event_index++];
            wasm_runner_call_on_input(&runner, (uint32_t)event.key, (uint32_t)event.type);
        }

        if (current_time == next_frame_time && frame_index < options.frames) {
            wasm_runner_call_render(&runner);
            frame_index++;
            next_frame_time = (frame_index < options.frames) ? frame_index * frame_ms : UINT32_MAX;
        }

        next_event_time = (next_event_index < timed_event_count)
            ? timed_events[next_event_index].time_ms
            : UINT32_MAX;
    }

    const char* app_id = options.app_id;
    if (!app_id) {
        const char* slash = strrchr(options.wasm_path, '/');
        app_id = slash ? slash + 1 : options.wasm_path;
    }

    if (!trace_write_json(options.output_path, app_id, options.seed, options.frames)) {
        fprintf(stderr, "Failed to write trace output\n");
        wasm_runner_deinit(&runner);
        free(timed_events);
        free(raw_events);
        return 1;
    }

    wasm_runner_deinit(&runner);
    free(timed_events);
    free(raw_events);
    return 0;
}
