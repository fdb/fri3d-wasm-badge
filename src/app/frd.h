#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Assert macro - halts on failure
#define frd_assert(expr) do { if (!(expr)) { frd_crash("Assert failed: " #expr); } } while(0)

// Crash function (implemented in frd_os.c)
void frd_crash(const char* message);

// Unused macro
#define UNUSED(x) (void)(x)
