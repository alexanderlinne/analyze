#pragma once

typedef struct report report_t;

#ifdef __cplusplus
extern "C" {
#endif

void report_add(report_t *, const char *output);

#ifdef __cplusplus
} // extern "C"
#endif

extern report_t *tracker_report;
