#include <stdlib.h>

#include "libc.h"
#include "report.h"
#include "util.h"

int access(const char *pathname, int mode)
{
    int result = real_access(pathname, mode);

    const char *json = safe_sprintf(
        "{\"type\":\"libc_call\",\"pid\":%ld,\"ppid\":%ld,\"timestamp\":%zu,"
        "\"function_name\":\"access\",\"path\":\"%s\",\"mode\":%d,\"result\":%d}\n",
        (long)getpid(), (long)getppid(), microseconds_since_epoch(),
        pathname, mode, result);
    report_add(tracker_report, json);
    free((void *)json);

    return result;
}
