#include <app.h>
#include <canvas.h>
#include <frd.h>
#include <input.h>
#include <random.h>

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define MAX_SNAKE_LEN 253
#define BOARD_MAX_X 30
#define BOARD_MAX_Y 14
#define STEP_INTERVAL_MS 250

typedef struct {
    uint8_t x;
    uint8_t y;
} point_t;

typedef enum {
    game_state_life,
    game_state_last_chance,
    game_state_game_over,
} game_state_t;

// Note: do not change without purpose. Current values are used in
// orthogonality calculation in snake_get_turn().
typedef enum {
    direction_up,
    direction_right,
    direction_down,
    direction_left,
} direction_t;

typedef struct {
    point_t points[MAX_SNAKE_LEN];
    uint16_t len;
    direction_t current_movement;
    direction_t next_movement;
    point_t fruit;
    game_state_t state;
    uint32_t last_step_ms;
    bool initialized;
} snake_state_t;

static snake_state_t g_snake;

static void snake_init_game(snake_state_t* snake) {
    static const point_t initial[] = {
        {8, 6}, {7, 6}, {6, 6}, {5, 6}, {4, 6}, {3, 6}, {2, 6},
    };

    memcpy(snake->points, initial, sizeof(initial));
    snake->len = 7;
    snake->current_movement = direction_right;
    snake->next_movement = direction_right;
    snake->fruit = (point_t){18, 6};
    snake->state = game_state_life;
    snake->last_step_ms = get_time_ms();
    snake->initialized = true;
}

static void snake_ensure_initialized(void) {
    if (!g_snake.initialized) {
        snake_init_game(&g_snake);
    }
}

static point_t snake_get_new_fruit(const snake_state_t* snake) {
    uint16_t buffer[8];
    memset(buffer, 0, sizeof(buffer));
    uint16_t empty = 8 * 16;

    for (uint16_t i = 0; i < snake->len; i++) {
        point_t p = snake->points[i];

        if ((p.x % 2) != 0 || (p.y % 2) != 0) {
            continue;
        }
        p.x /= 2;
        p.y /= 2;

        buffer[p.y] |= (uint16_t)(1u << p.x);
        empty--;
    }

    if (empty == 0) {
        return (point_t){0, 0};
    }

    uint16_t new_fruit = (uint16_t)random_range(empty);

    for (uint8_t y = 0; y < 8; y++) {
        for (uint16_t x = 0, mask = 1; x < 16; x++, mask <<= 1) {
            if ((buffer[y] & mask) == 0) {
                if (new_fruit == 0) {
                    return (point_t){(uint8_t)(x * 2), (uint8_t)(y * 2)};
                }
                new_fruit--;
            }
        }
    }

    return (point_t){0, 0};
}

static bool snake_collision_with_frame(point_t next_step) {
    return (next_step.x > BOARD_MAX_X) || (next_step.y > BOARD_MAX_Y);
}

static bool snake_collision_with_tail(const snake_state_t* snake, point_t next_step) {
    for (uint16_t i = 0; i < snake->len; i++) {
        point_t p = snake->points[i];
        if (p.x == next_step.x && p.y == next_step.y) {
            return true;
        }
    }

    return false;
}

static direction_t snake_get_turn(const snake_state_t* snake) {
    bool is_orthogonal = ((snake->current_movement + snake->next_movement) % 2) == 1;
    return is_orthogonal ? snake->next_movement : snake->current_movement;
}

static point_t snake_get_next_step(const snake_state_t* snake) {
    point_t next_step = snake->points[0];

    switch (snake->current_movement) {
        case direction_up:
            next_step.y--;
            break;
        case direction_right:
            next_step.x++;
            break;
        case direction_down:
            next_step.y++;
            break;
        case direction_left:
            next_step.x--;
            break;
        default:
            break;
    }

    return next_step;
}

static void snake_move_snake(snake_state_t* snake, point_t next_step) {
    memmove(snake->points + 1, snake->points, snake->len * sizeof(point_t));
    snake->points[0] = next_step;
}

static void snake_process_step(snake_state_t* snake) {
    if (snake->state == game_state_game_over) {
        return;
    }

    bool can_turn = ((snake->points[0].x % 2) == 0) && ((snake->points[0].y % 2) == 0);
    if (can_turn) {
        snake->current_movement = snake_get_turn(snake);
    }

    point_t next_step = snake_get_next_step(snake);

    bool crash = snake_collision_with_frame(next_step);
    if (crash) {
        if (snake->state == game_state_life) {
            snake->state = game_state_last_chance;
            return;
        }
        if (snake->state == game_state_last_chance) {
            snake->state = game_state_game_over;
            return;
        }
    } else if (snake->state == game_state_last_chance) {
        snake->state = game_state_life;
    }

    if (snake_collision_with_tail(snake, next_step)) {
        snake->state = game_state_game_over;
        return;
    }

    bool eat_fruit = (next_step.x == snake->fruit.x) && (next_step.y == snake->fruit.y);
    if (eat_fruit) {
        snake->len++;
        if (snake->len >= MAX_SNAKE_LEN) {
            snake->state = game_state_game_over;
            return;
        }
    }

    snake_move_snake(snake, next_step);

    if (eat_fruit) {
        snake->fruit = snake_get_new_fruit(snake);
    }
}

static void snake_update(void) {
    uint32_t now_ms = get_time_ms();
    uint32_t elapsed = now_ms - g_snake.last_step_ms;

    while (elapsed >= STEP_INTERVAL_MS) {
        snake_process_step(&g_snake);
        g_snake.last_step_ms += STEP_INTERVAL_MS;
        elapsed = now_ms - g_snake.last_step_ms;
    }
}

static void snake_render(const snake_state_t* snake) {
    canvas_set_color(ColorBlack);

    canvas_draw_frame(0, 0, 128, 64);

    point_t fruit = snake->fruit;
    int32_t fruit_x = fruit.x * 4 + 1;
    int32_t fruit_y = fruit.y * 4 + 1;
    canvas_draw_rframe(fruit_x, fruit_y, 6, 6, 2);

    for (uint16_t i = 0; i < snake->len; i++) {
        point_t p = snake->points[i];
        int32_t px = p.x * 4 + 2;
        int32_t py = p.y * 4 + 2;
        canvas_draw_box(px, py, 4, 4);
    }

    if (snake->state == game_state_game_over) {
        canvas_set_color(ColorWhite);
        canvas_draw_box(34, 20, 62, 24);

        canvas_set_color(ColorBlack);
        canvas_draw_frame(34, 20, 62, 24);

        canvas_set_font(FontPrimary);
        canvas_draw_str(37, 31, "Game Over");

        canvas_set_font(FontSecondary);
        char buffer[16];
        snprintf(buffer, sizeof(buffer), "Score: %u", (unsigned)(snake->len - 7U));
        uint32_t width = canvas_string_width(buffer);
        int32_t x = (128 - (int32_t)width) / 2;
        canvas_draw_str(x, 41, buffer);
    }
}

void render(void) {
    snake_ensure_initialized();
    snake_update();
    snake_render(&g_snake);
}

void on_input(input_key_t key, input_type_t type) {
    snake_ensure_initialized();

    if (key == input_key_back) {
        if (type == input_type_short_press || type == input_type_long_press) {
            exit_to_launcher();
        }
        return;
    }

    if (type != input_type_press) {
        return;
    }

    if (key == input_key_ok) {
        if (g_snake.state == game_state_game_over) {
            snake_init_game(&g_snake);
        }
        return;
    }

    if (g_snake.state == game_state_game_over) {
        return;
    }

    switch (key) {
        case input_key_up:
            g_snake.next_movement = direction_up;
            break;
        case input_key_down:
            g_snake.next_movement = direction_down;
            break;
        case input_key_left:
            g_snake.next_movement = direction_left;
            break;
        case input_key_right:
            g_snake.next_movement = direction_right;
            break;
        default:
            break;
    }
}

int main(void) {
    return 0;
}
