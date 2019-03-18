#define _GNU_SOURCE

#include "file_report.h"

#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <unistd.h>

#include "libc.h"
#include "util.h"

static int _argc;
static char **_argv;
static char **_environ;

struct file_descriptor {
    pid_t process_id;
    pid_t thread_id;
    int file_descriptor;
};

struct file_report {
    void (*add)(report_t *, const char *output);
    char *output_path;
    size_t json_values_size;
    size_t json_values_idx;
    char **json_values;
};

void file_report_add(report_t *_report, const char *output);

file_report_t *file_report_create(int argc, char **argv, const char *output_path)
{
    _argc = argc;
    _argv = argv;
    _environ = environ;
    file_report_t *_file_report = malloc(sizeof(file_report_t));
    _file_report->add = file_report_add;
    _file_report->output_path = malloc(strlen(output_path));
    strcpy(_file_report->output_path, output_path);
    _file_report->json_values_size = 1;
    _file_report->json_values_idx = 0;
    _file_report->json_values = malloc(sizeof(char *));
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
    if (_file_report->json_values_idx + 1 >= _file_report->json_values_size) {
        _file_report->json_values_size =
            _file_report->json_values_size *= 2;
        _file_report->json_values = realloc(_file_report->json_values,
            _file_report->json_values_size * sizeof(char *));
    }
    size_t str_length = strlen(output);
    char *buffer = malloc(str_length + 1);
    memcpy(buffer, output, str_length);
    buffer[str_length] = '\0';
    _file_report->json_values[_file_report->json_values_idx++] = buffer;
}

void file_report_destroy(file_report_t *_file_report)
{
    real_mkdir(_file_report->output_path, 0700);
    const char *cfp = file_report_current_filepath(_file_report);
    int file = real_open(cfp, O_WRONLY | O_CREAT | O_APPEND, 0600);
    free((void *)cfp);

    _file_report->json_values[_file_report->json_values_idx] = NULL;
    const char *log = json_value_array_to_json_list(
        (const char * const *)_file_report->json_values);

    const char *json = safe_sprintf(
        "{"
            "\"pid\":%ld,"
            "\"ppid\":%ld,"
            "\"argv\":%s,"
            "\"envp\":%s,"
            "\"log\":%s"
        "}\n",
        (long)getpid(), (long)getppid(),
        string_array_to_json_list((const char * const *)_argv),
        string_array_to_json_list((const char * const *)_environ),
        log);
    free((void *)log);
    real_write(file, json, strlen(json));
    free((void *)json);

    free(_file_report->output_path);
}
