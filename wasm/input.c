#include "blaulicht.h"

#define IS_WHITE_VALUE_INDEX 7
#define WHITE_VALUE_INDEX 8 

#define BPM_STROBE_LAST_TICK_INDEX 9
#define BPM_STROBE_LAST_TICK_HIGH_INDEX 10
#define BPM_STROBE_MODE_INDEX 11

#define BPM_STROBE_DURATION_MILLIS 10
#define STROBE_TIME 60

#define STROBE_START_INDEX 200
#define IS_STROBE_INDEX 201

#define MAX_BRIGHTNESS_NON_STROBE 50

int abs(int32_t x) {
    return (x < 0) ? -x : x;
}


int elapsed(int32_t time, int32_t * data, int32_t index) {
    return time - data[index];
}

void write_last_bpm_flash(int32_t *data, int32_t time) {
    data[BPM_STROBE_LAST_TICK_INDEX] = time;
}

void write_last_bpm_flash_high(int32_t *data, int32_t time) {
    data[BPM_STROBE_LAST_TICK_HIGH_INDEX] = time;
}

int since_last_bpm_flash(int32_t *data, int32_t time) {
    return elapsed(time, data, BPM_STROBE_LAST_TICK_INDEX);
}

int since_last_bpm_flash_high(int32_t *data, int32_t time) {
    return elapsed(time, data, BPM_STROBE_LAST_TICK_HIGH_INDEX);
}

void set_white(int v, uint8_t * dmx) {
    dmx[1] = v;
    dmx[2] = v;
    dmx[3] = v;
    dmx[4] = v;

    dmx[100] = v;
    dmx[101] = v;
    dmx[102] = v;
}

// int pow(int base, int exp) {
//     int acc = base;
//     for (int i = 1; i < exp; i++) {
//         acc *= base;
//     }
//     return acc;
// }

long map(int32_t x, int32_t in_min, int32_t in_max, int32_t out_min, int32_t out_max) {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

// Convert HSV hue (0–360) to RGB
void hsv_to_rgb(int32_t h, int32_t *r, int32_t *g, int32_t *b) {
    float s = 1.0f;
    float v = 1.0f;
    float c = v * s;
    float x = c * (1 - abs((h / 60 % 2) - 1));
    float m = v - c;

    float rf = 0, gf = 0, bf = 0;

    if (h < 60) {
        rf = c; gf = x; bf = 0;
    } else if (h < 120) {
        rf = x; gf = c; bf = 0;
    } else if (h < 180) {
        rf = 0; gf = c; bf = x;
    } else if (h < 240) {
        rf = 0; gf = x; bf = c;
    } else if (h < 300) {
        rf = x; gf = 0; bf = c;
    } else {
        rf = c; gf = 0; bf = x;
    }

    *r = (int32_t)((rf + m) * 255);
    *g = (int32_t)((gf + m) * 255);
    *b = (int32_t)((bf + m) * 255);
}

void initialize(TickInput input, uint8_t *dmx_array, int32_t dmx_len) {
    bl_puts("WASM: Initialized was called.");
}

void tick(
    TickInput input,
    uint8_t *dmx, int32_t dmx_len,
    int32_t *data, int32_t data_len
) {
    // --- Rainbow RGB effect ---
    if (since_last_bpm_flash(data, input.time) > 1000) {
        int hue = data[150]; // persistent hue
        int r, g, b;
        hsv_to_rgb(hue, &r, &g, &b);

        // scale brightness with volume
        r = r;
        g = g;
        b = b;

        dmx[1] = input.volume;
        dmx[2] = r;
        dmx[3] = g;
        dmx[4] = b;

        // TODO: advance hue
        // if (input.volume > 0) {
        //     hue = (hue + 1) % 360;
        //     data[150] = hue;
        // }
    }

    // --- Audio-reactive white strobe ---
    int time_since_strobe_start = elapsed(input.time, data, STROBE_START_INDEX);

    if (
        (input.bass_avg < 50 && input.bass > 100 && !data[IS_STROBE_INDEX]) ||
        (time_since_strobe_start < STROBE_TIME && data[IS_STROBE_INDEX]) &&
        input.volume > 150) {
        if (!data[IS_STROBE_INDEX]) {
            data[STROBE_START_INDEX] = input.time;
        }

        data[WHITE_VALUE_INDEX] = data[IS_WHITE_VALUE_INDEX] ? 0 : 255;
        data[IS_WHITE_VALUE_INDEX] = !data[IS_WHITE_VALUE_INDEX];

        set_white(data[WHITE_VALUE_INDEX], dmx);
        // bl_puts("Entered strobe branch.");
        return;
    } else if (data[IS_WHITE_VALUE_INDEX]) {
        set_white(0, dmx);
        data[IS_WHITE_VALUE_INDEX] = 0;
        data[IS_STROBE_INDEX] = 0;
    }

    // --- BPM-based strobe ---
    int elapsed = since_last_bpm_flash(data, input.time);
    int bpm = input.bpm;
    int target_elapsed_bpm = (int)((1.0 / (float)bpm) * 60.0 * 1000.0 * 1.0);

    if (input.bass_avg < 100 && input.bass < 100) {
        if (data[IS_WHITE_VALUE_INDEX]) {
            bl_puts("Entered set-dark branch.");
            data[WHITE_VALUE_INDEX] = 0;
            data[IS_WHITE_VALUE_INDEX] = 0;
            set_white(data[WHITE_VALUE_INDEX], dmx);
        }
    } else if (elapsed > target_elapsed_bpm && !data[IS_WHITE_VALUE_INDEX]) {
        write_last_bpm_flash(data, input.time);
        data[IS_WHITE_VALUE_INDEX] = 1;
        data[WHITE_VALUE_INDEX] = 255;
        set_white(data[WHITE_VALUE_INDEX], dmx);
    } else if (data[IS_WHITE_VALUE_INDEX]) {
        write_last_bpm_flash_high(data, input.time);
        data[WHITE_VALUE_INDEX] = 0;
        data[IS_WHITE_VALUE_INDEX] = 0;
        set_white(data[WHITE_VALUE_INDEX], dmx);
    }
}
