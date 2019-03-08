#include "report.h"

struct report {
    void (*add)(report_t *, const char *output);
};

void report_add(report_t *_report, const char *output) {
    _report->add(_report, output);
}

report_t *tracker_report;
