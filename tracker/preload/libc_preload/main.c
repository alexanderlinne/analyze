#define _GNU_SOURCE

#include <stdlib.h>
#include <unistd.h>

#include "file_report.h"
#include "libc.h"
#include "util.h"

static file_report_t *_file_report = NULL;

__attribute__((constructor))
static void __constructor(int argc, char **argv) {
    init_libc_function_pointers();
    const char *output_path = getenv("TRACKER_OUTPUT_PATH");
    if (output_path) {
        tracker_report = (report_t *)file_report_create(
            argc, argv, output_path);
    } else {
        tracker_report = (report_t *)file_report_create(
            argc, argv, get_current_dir_name());
    }

    const char *json = safe_sprintf(
        "{"
            "\"type\":\"preload_loaded\","
            "\"timestamp\":%zu,"
            "\"data\":{}"
        "}",
        microseconds_since_epoch());
    report_add(tracker_report, json);
    free((void *)json);
}

__attribute__((destructor))
static void __destructor() {
    const char *json = safe_sprintf(
        "{"
            "\"type\":\"preload_unloaded\","
            "\"timestamp\":%zu,"
            "\"data\":{}"
        "}",
        microseconds_since_epoch());
    report_add(tracker_report, json);
    free((void *)json);

    file_report_destroy((file_report_t *)tracker_report);
    tracker_report = NULL;
}
