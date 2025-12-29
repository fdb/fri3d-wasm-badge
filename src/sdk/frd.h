#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Unused macro to suppress warnings
#define UNUSED(x) (void)(x)

// WASM import helper macro
#define WASM_IMPORT(name) __attribute__((import_module("env"), import_name(name)))

// Simple string utilities (no libc dependency)

/**
 * Convert signed integer to string.
 * Returns pointer to end of written string.
 */
static inline char* frd_itoa(int32_t value, char* buf) {
    char* p = buf;
    uint32_t uval;

    if (value < 0) {
        *p++ = '-';
        uval = (uint32_t)(-(value + 1)) + 1;
    } else {
        uval = (uint32_t)value;
    }

    // Write digits in reverse
    char* start = p;
    do {
        *p++ = '0' + (uval % 10);
        uval /= 10;
    } while (uval);

    // Reverse the digits
    char* end = p;
    p--;
    while (start < p) {
        char tmp = *start;
        *start++ = *p;
        *p-- = tmp;
    }

    *end = '\0';
    return end;
}

/**
 * Get string length.
 */
static inline size_t frd_strlen(const char* s) {
    const char* p = s;
    while (*p) p++;
    return p - s;
}

/**
 * Copy string, returns pointer to end of dest.
 */
static inline char* frd_strcpy(char* dest, const char* src) {
    while (*src) *dest++ = *src++;
    *dest = '\0';
    return dest;
}
