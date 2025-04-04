#include "blaulicht.h"

// NOTE: this file is completely overwritten by the reef compiler.
// It only exists so that we don't get errors during development.

#define IS_WHITE_VALUE_INDEX 7
#define WHITE_VALUE_INDEX 8 

#define BPM_STROBE_LAST_TICK_INDEX 9
#define BPM_STROBE_LAST_TICK_HIGH_INDEX 10
#define BPM_STROBE_MODE_INDEX 11

int elapsed(int time, int * data, int index) {
    int elapsed = time - data[index];
    return elapsed;
}

void write_last_bpm_flash(int *data, int time) {
    data[BPM_STROBE_LAST_TICK_INDEX] = time;
}

void write_last_bpm_flash_high(int *data, int time) {
    data[BPM_STROBE_LAST_TICK_HIGH_INDEX] = time;
}

int since_last_bpm_flash(int *data, int time) {
    return elapsed(time, data, BPM_STROBE_LAST_TICK_INDEX);
}

int since_last_bpm_flash_high(int *data, int time) {
    return elapsed(time, data, BPM_STROBE_LAST_TICK_HIGH_INDEX);
}

void set_white(int v, int * dmx) {
    dmx[1] = v;
    dmx[2] = v;
    dmx[3] = v;
    dmx[4] = v;

    dmx[100] = v;
    dmx[101] = v;
    dmx[102] = v;
}

#define BPM_STROBE_DURATION_MILLIS 20


void tick(
    TickInput input,
    int *dmx, int dmx_len,
    int *data, int data_len
) {
    // dmx[1] = 255;
    // dmx[2] = 255;
    // dmx[3] = 255;
    // dmx[4] = 255;

    // return;

   // int al = elapsed(input.time, data, 123);
    // data[123] = input.time;
    // bl_log_int(al);

    // bl_log_int(input.volume);

    // Red flashing.
    dmx[1] = input.volume;
    dmx[2] = 255;
    dmx[3] = 0;
    // dmx[0] = 0;
    // dmx[0] = 0;

    if (input.bass_avg < 30 && input.bass > 100) {
        if (data[IS_WHITE_VALUE_INDEX]) {
            data[WHITE_VALUE_INDEX] = 255;
        } else {
            data[WHITE_VALUE_INDEX] = 0;
        }
        data[IS_WHITE_VALUE_INDEX] = !data[IS_WHITE_VALUE_INDEX];
        set_white(data[WHITE_VALUE_INDEX], dmx);
        return;
    } else if (data[IS_WHITE_VALUE_INDEX]) {
        set_white(0, dmx);
        data[IS_WHITE_VALUE_INDEX] = 0;
    }

    return;

    ///
    /// BPM STROBE
    ///

    int elapsed = since_last_bpm_flash(data, input.time);
    // bl_puts("DIFF:");
    // bl_log_int(elapsed);
    // bl_puts("BPM:");
    // bl_log_int(input.bpm);

    // input.bpm = 130;
    // input.bass = 100;
    int bass_acc = input.bass_avg;
    int bass = input.bass;

    // bass = 100;

    int bpm = input.bpm;
    // bpm = 130;
    int target_elapsed_bpm = (int) ( (float) 1 / (float) bpm * 60.0 * 1000.0);

    // #define WHITE data[WHITE_VALUE_INDEX]
    // #define IS_WHITE data[IS_WHITE_VALUE_INDEX]

    // int elapsed_since_bpm_high = since_last_bpm_flash_high(data, input.time);
    // bl_log_int(elapsed_since_bpm_high);
    // bl_log_int(elapsed);
    if (data[99] != data[IS_WHITE_VALUE_INDEX]) {
        // bl_log_int(data[IS_WHITE_VALUE_INDEX]);
        data[99] = data[IS_WHITE_VALUE_INDEX];
    }

    if (elapsed > target_elapsed_bpm) {
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

    if (bass_acc < 100 && bass < 100 && data[IS_WHITE_VALUE_INDEX]) {
        data[WHITE_VALUE_INDEX] = 0;
        data[IS_WHITE_VALUE_INDEX] = 0;
        set_white(data[WHITE_VALUE_INDEX], dmx);
    }
}
