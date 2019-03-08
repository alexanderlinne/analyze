#define _GNU_SOURCE

#include <stdlib.h>
#include <unistd.h>

#include "file_report.h"
#include "libc.h"

static file_report_t *_file_report = NULL;

__attribute__((constructor))
static void __constructor() {
    init_libc_function_pointers();
    const char *output_path = getenv("TRACKER_OUTPUT_PATH");
    if (output_path) {
        tracker_report = (report_t *)file_report_create(output_path);
    } else {
        tracker_report = (report_t *)file_report_create(get_current_dir_name());
    }
}

__attribute__((destructor))
static void __destructor() {
    file_report_destroy((file_report_t *)tracker_report);
    tracker_report = NULL;
}
