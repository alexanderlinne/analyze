#pragma once

#include "report.h"

typedef struct file_descriptor file_descriptor_t;
typedef struct file_report file_report_t;

#ifdef __cplusplus
extern "C" {
#endif

file_report_t *file_report_create(int argc, char **argv, const char *output_path);
const char *file_report_current_filepath(file_report_t *);
void file_report_destroy(file_report_t *);

#ifdef __cplusplus
} // extern "C"
#endif
