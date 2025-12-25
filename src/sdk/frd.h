#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Unused macro to suppress warnings
#define UNUSED(x) (void)(x)

// WASM import helper macro
#define WASM_IMPORT(name) __attribute__((import_module("env"), import_name(name)))
