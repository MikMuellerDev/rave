#include "blaulicht.h"

// Internal imports
#include "imports.h"

// #include "input.c"

//
// Wasm entry point
//
// void blaulicht_main() {
//     size_t len = _reef_dataset_len();
//     size_t len_alloc = (len + 7) & ~0x07;

//     uint8_t *dataset_mem = malloc(len_alloc);
//     _reef_dataset_write(dataset_mem);

//     reef_result_int(0);

//     run(dataset_mem, len);
// }

// Result functions
// void reef_result_int(int value) {
//     uint8_t *ptr = (uint8_t *)&value;
//     _reef_result(0, ptr, 4);
// }
// void reef_result_bytes(uint8_t *ptr, size_t len) { _reef_result(1, ptr, len); }
// void reef_result_string(char *ptr, size_t len) { _reef_result(2, (uint8_t *)ptr, len); }

// Conversion functions
// uint32_t *from_little_endian(uint8_t *arr, size_t n_bytes) {
//     uint32_t *ret = (uint32_t *)malloc(n_bytes);

//     for (int i = 0; i < n_bytes / 4; i++) {
//         ret[i] = (arr[i * 4 + 3] << 24) + (arr[i * 4 + 2] << 16) + (arr[i * 4 + 1] << 8) + arr[i * 4];
//     }

//     return ret;
// }

// uint8_t *to_little_endian(uint32_t *arr, size_t n_bytes) {
//     uint8_t *ret = (uint8_t *)malloc(n_bytes);

//     for (int i = 0; i < n_bytes / 4; i++) {
//         ret[i * 4] = arr[i] & 0xff;
//         ret[i * 4 + 1] = (arr[i] >> 8) & 0xff;
//         ret[i * 4 + 2] = (arr[i] >> 16) & 0xff;
//         ret[i * 4 + 3] = (arr[i] >> 24) & 0xff;
//     }

//     return ret;
// }

// uint32_t *from_big_endian(uint8_t *arr, size_t n_bytes) {
//     uint32_t *ret = (uint32_t *)malloc(n_bytes);

//     for (int i = 0; i < n_bytes / 4; i++) {
//         ret[i] = (arr[i * 4] << 24) + (arr[i * 4 + 1] << 16) + (arr[i * 4 + 2] << 8) + arr[i * 4 + 3];
//     }

//     return ret;
// }

// uint8_t *to_big_endian(uint32_t *arr, size_t n_bytes) {
//     uint8_t *ret = (uint8_t *)malloc(n_bytes);

//     for (int i = 0; i < n_bytes / 4; i++) {
//         ret[i * 4 + 3] = arr[i] & 0xff;
//         ret[i * 4 + 2] = (arr[i] >> 8) & 0xff;
//         ret[i * 4 + 1] = (arr[i] >> 16) & 0xff;
//         ret[i * 4] = (arr[i] >> 24) & 0xff;
//     }

//     return ret;
// }

TickInput tickinput_from_array(int * tick_input_array, int tick_array_len) {
    #define ARRAY_LEN 7

    if (tick_array_len != ARRAY_LEN) {
        bl_puts("tick array len in 'tickinput_from_array' is not expected length:");
        bl_log_int(tick_array_len);
        abort();
    }

    TickInput input = {
        .time = tick_input_array[0],
        .volume = tick_input_array[1],
        .beat_volume = tick_input_array[2],
        .bass = tick_input_array[3],
        .bass_avg = tick_input_array[4],
        .bpm = tick_input_array[5],
        .initial = tick_input_array[6],
    };

    return input;
}

void internal_tick(
    int32_t * tick_input_array, int32_t tick_array_len,
    uint8_t * dmx_array, int32_t dmx_array_len,
    int32_t * data_array, int32_t data_len
) {
    TickInput input = tickinput_from_array(tick_input_array, tick_array_len);

    #define DMX_LEN 513

    if (dmx_array_len != DMX_LEN) {
        bl_puts("DMX array in 'internal_tick()' is not of exected length; got:");
        bl_log_int(dmx_array_len);
        bl_puts("expected: ");
        bl_log_int(DMX_LEN);
        abort();
    }

    if (input.initial) {
        initialize(input, dmx_array, dmx_array_len);
    } else {
        tick(input, dmx_array, dmx_array_len, data_array, data_len);
    }
}

void abort() {
    bl_puts("called abort()");
    while (1) {
    }
}