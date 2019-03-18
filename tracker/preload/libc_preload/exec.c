#include <stdlib.h>
#include <string.h>

#include "libc.h"
#include "report.h"
#include "util.h"

void exit_called(
    int status)
{
    const char *json = safe_sprintf(
        "{"
            "\"type\":\"libc_call\","
            "\"timestamp\":%zu,"
            "\"data\":{"
                "\"function_name\":\"exit\","
                "\"status\":%d"
            "}"
        "}",
        microseconds_since_epoch(), status);
    report_add(tracker_report, json);
    free((void *)json);
}

void exec_called(
    const char *function_name,
    const char *filename,
    const char * const *argv,
    const char * const *envp)
{
    const char *argv_json = string_array_to_json_list(argv);
    const char *envp_json = string_array_to_json_list(envp);
    const char *json = safe_sprintf(
        "{"
            "\"type\":\"libc_call\","
            "\"timestamp\":%zu,"
            "\"data\":{"
                "\"function_name\":\"%s\","
                "\"filename\":\"%s\","
                "\"argv\":%s,"
                "\"envp\":%s"
            "}"
        "}",
        microseconds_since_epoch(),
        function_name, filename, argv_json, envp_json);
    report_add(tracker_report, json);
    free((void *)json);
    free((void *)envp_json);
    free((void *)argv_json);
}

void exit(int status) {
    exit_called(status);
    real_exit(status);
    __builtin_unreachable();
}

void _exit(int status) {
    exit_called(status);
    real__exit(status);
    __builtin_unreachable();
}

void _Exit(int status) {
    exit_called(status);
    real__Exit(status);
    __builtin_unreachable();
}

int execve(const char *filename, char *const argv[], char *const envp[])
{
    exec_called("execve", filename,
        (const char * const *)argv,
        (const char * const *)envp);
    return real_execve(filename, argv, envp);
}

int execl(const char *path, const char *arg, ... /* (char  *) NULL */)
{
    char *const *argv = (char **)arg;
    char *envp[1] = {NULL};
    exec_called("execl", path,
        (const char * const *)argv, (const char * const *)envp);
    return real_execv(path, argv);
}

int execlp(const char *file, const char *arg, ... /* (char  *) NULL */)
{
    char *const *argv = (char **)arg;
    char *envp[1] = {NULL};
    exec_called("execlp", file,
        (const char * const *)argv, (const char * const *)envp);
    return real_execvp(file, argv);
}

int execle(const char *path, const char *arg, ...
    /*, (char *) NULL, char * const envp[] */)
{
    char *const *argv = (char **)arg;
    // Seek to the end of argv.
    while (++arg);
    // The end of argv is directly followed by envp.
    char *const *envp = (char **)(arg + 1);
    exec_called("execle", path,
        (const char * const *)argv,
        (const char * const *)envp);
    return real_execve(path, argv, envp);
}

int execv(const char *path, char *const argv[])
{
    char *envp[1] = {NULL};
    exec_called("execv", path,
        (const char * const *)argv, (const char * const *)envp);
    return real_execv(path, argv);
}

int execvp(const char *file, char *const argv[])
{
    char *envp[1] = {NULL};
    exec_called("execvp", file,
        (const char * const *)argv, (const char * const *)envp);
    return real_execvp(file, argv);
}

int execvpe(const char *file, char *const argv[], char *const envp[])
{
    exec_called("execvpe", file,
        (const char * const *)argv,
        (const char * const *)envp);
    return real_execvpe(file, argv, (const char * const *)envp);
}

int fexecve(int fd, char *const argv[], char *const envp[]) {
    const char *linkname = linkname_of(fd);
    exec_called("fexecve",
        linkname,
        (const char * const *)argv,
        (const char * const *)envp);
    free((void *)linkname);
    return real_fexecve(fd, argv, (const char * const *)envp);
}
