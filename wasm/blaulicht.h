#pragma once

#include "defs.h"
#include "log.h"
#include "walloc.h"
#include "memory.h"

// Wasm Function Imports.
// void reef_progress(float done) __attribute__((__import_module__("reef"), __import_name__("progress"), ));
// void reef_sleep(float seconds) __attribute__((__import_module__("reef"), __import_name__("sleep"), ));

// User main function definition
// void run(uint8_t *dataset, size_t len);

typedef struct {
    uint64_t time;
    uint8_t volume;
    uint8_t beat_volume;
    uint8_t bass;
    uint8_t bass_avg;
    uint8_t bpm;
    bool initial;
} TickInput;

void internal_tick(
    int32_t * tick_input_array, int32_t tick_array_len,
    uint8_t * dmx_array, int32_t dmx_array_len,
    int32_t * data_array, int32_t data_len
);

void initialize(TickInput input, uint8_t *dmx_array, int32_t dmx_array_len);

void tick(
    TickInput input,
     uint8_t * dmx_array, int32_t dmx_array_len,
     int32_t *data, int32_t data_len
);

// Result functions
// void reef_result_int(int value);
// void reef_result_bytes(uint8_t *ptr, size_t len);
// void reef_result_string(char *ptr, size_t len);

// Conversion functions
// uint32_t *from_little_endian(uint8_t *arr, size_t n_bytes);
// uint8_t *to_little_endian(uint32_t *arr, size_t n_bytes);
// uint32_t *from_big_endian(uint8_t *arr, size_t n_bytes);
// uint8_t *to_big_endian(uint32_t *arr, size_t n_bytes);

void abort();