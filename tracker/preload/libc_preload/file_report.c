#include "file_report.h"

#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <unistd.h>

#include "libc.h"
#include "util.h"

struct file_descriptor {
    pid_t process_id;
    pid_t thread_id;
    int file_descriptor;
};

struct file_report {
    void (*add)(report_t *, const char *output);
    char *output_path;
    int file;
};

void file_report_add(report_t *_report, const char *output);

file_report_t *file_report_create(const char *output_path)
{
    file_report_t *_file_report = malloc(sizeof(file_report_t));
    _file_report->add = file_report_add;
    _file_report->output_path = malloc(strlen(output_path));
    strcpy(_file_report->output_path, output_path);
    _file_report->file = -1;
}

const char *file_report_current_filepath(file_report_t *_file_report)
{
    pid_t pid = getpid();
    pid_t tid = syscall(SYS_gettid);
    if (pid == tid) {
        return safe_sprintf("%s/log_%ld.json",
            _file_report->output_path, (long)pid);
    } else {
        return safe_sprintf("%s/log_%ld_%ld.json",
            _file_report->output_path, (long)pid, (long)tid);
    }
}

void file_report_add(report_t *_report, const char *output) {
    file_report_t *_file_report = (file_report_t *)_report;
    if (_file_report->file == -1) {
        real_mkdir(_file_report->output_path, 0700);
        const char *cfp = file_report_current_filepath(_file_report);
        _file_report->file = real_open(cfp, O_WRONLY | O_CREAT | O_APPEND, 0600);
        free((void *)cfp);
    }
    real_write(_file_report->file, output, strlen(output));
}

void file_report_destroy(file_report_t *_file_report)
{
    free(_file_report->output_path);
}
