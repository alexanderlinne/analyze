#define _GNU_SOURCE

#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>

#include "file_report.h"
#include "libc.h"
#include "util.h"

static int _argc;
static char **_argv;

void set_up()
{
    init_libc_function_pointers();
}

bool test_file_report()
{
    file_report_t *report = file_report_create(
        _argc, _argv, get_current_dir_name());
    report_add((report_t *)report, "test0");
    report_add((report_t *)report, "test1");
    const char *report_path = file_report_current_filepath(report);
    file_report_destroy(report);
    FILE *file = NULL;
    char *file_contents = NULL;
    const char *expected = NULL;

    bool success = true;
    if (access(report_path, F_OK) == -1) {
        success = false;
        goto _exit;
    }

    size_t len = 0;
    file = fopen(report_path, "r");
    if (file == NULL) {
        success = false;
        goto _exit;
    }

    if (getline(&file_contents, &len, file) == -1) {
        success = false;
        goto _exit;
    }

    expected = safe_sprintf(
        "{"
            "\"pid\":%ld,"
            "\"ppid\":%ld,"
            "\"argv\":%s,"
            "\"envp\":%s,"
            "\"log\":[test0,test1]"
        "}\n",
        (long)getpid(), (long)getppid(),
        string_array_to_json_list((const char * const *)_argv),
        string_array_to_json_list((const char * const *)environ));
    if (strcmp(file_contents, expected) != 0) {
        success = false;
        goto _exit;
    }

_exit:
    if (expected)
        free((void *)expected);
    if (file_contents)
        free(file_contents);
    if (file)
        fclose(file);
    free((void *)report_path);
    return success;
}

bool test_standard_linkname(int fd, const char *expected)
{
    const char *linkname = linkname_of(fd);
    bool success = strcmp(linkname, expected) == 0;
    free((void *)linkname);
    return success;
}

bool test_linkname_file()
{
    const char *filename = safe_sprintf("%s/test.txt", get_current_dir_name());
    int file_descriptor = open(filename, O_RDONLY | O_CREAT, 0600);

    const char *linkname = linkname_of(file_descriptor);
    bool success = strcmp(linkname, filename) == 0;

    free((void *)linkname);
    close(file_descriptor);
    remove(filename);
    free((void *)filename);

    return success;
}

bool test_bytes_to_hex_string()
{
    const char *test_string = "\x2b\x41\x7f\x80\xff";
    const char *result = bytes_to_hex_string(
        (void *)test_string,
        strlen(test_string));
    bool success = strcmp(result, "2b417f80ff") == 0;
    free((void *)result);
    return success;
}

bool test_empty_string_to_json_value() {
    const char *result = string_to_json_value("");
    bool success = strcmp(result, "\"\"") == 0;
    free((void *)result);
    return success;
}

bool test_simple_string_to_json_value() {
    const char *result = string_to_json_value("test");
    bool success = strcmp(result, "\"test\"") == 0;
    free((void *)result);
    return success;
}

bool test_string_to_json_value_escape_quotes() {
    const char *result = string_to_json_value("\"");
    bool success = strcmp(result, "\"\\\"\"") == 0;
    free((void *)result);
    return success;
}

bool test_string_to_json_value_escape_backslash() {
    const char *result = string_to_json_value("\\");
    bool success = strcmp(result, "\"\\\\\"") == 0;
    free((void *)result);
    return success;
}

bool test_string_to_json_value_escape_control_characters() {
    const char *result = string_to_json_value("\x07");
    bool success = strcmp(result, "\"\\u0007\"") == 0;
    free((void *)result);
    return success;
}

bool test_empty_array_to_json_array() {
    const char *strings[1];
    strings[0] = NULL;
    const char *result = string_array_to_json_list(strings);
    bool success = strcmp(result, "[]") == 0;
    free((void *)result);
    return success;
}

bool test_array_to_json_array() {
    const char *strings[3];
    strings[0] = "test0";
    strings[1] = "test1";
    strings[2] = NULL;
    const char *result = string_array_to_json_list(strings);
    bool success = strcmp(result, "[\"test0\",\"test1\"]") == 0;
    free((void *)result);
    return success;
}

bool test_array_to_json_array_escape() {
    const char *strings[2];
    strings[0] = "\\\"";
    strings[1] = NULL;
    const char *result = string_array_to_json_list(strings);
    bool success = strcmp(result, "[\"\\\\\\\"\"]") == 0;
    free((void *)result);
    return success;
}

#define TEST(command) \
    printf(#command); \
    if ( !command ) { \
        printf(" ... FAILED\n"); \
        success = false; \
    } else { \
        printf(" ... ok\n"); \
    }

int main(int argc, char **argv)
{
    _argc = argc;
    _argv = argv;
    set_up();
    bool success = true;
    TEST(test_file_report())
    TEST(test_standard_linkname(0, "stdin"))
    TEST(test_standard_linkname(1, "stdout"))
    TEST(test_standard_linkname(2, "stderr"))
    TEST(test_linkname_file())
    TEST(test_bytes_to_hex_string())
    TEST(test_empty_string_to_json_value())
    TEST(test_simple_string_to_json_value())
    TEST(test_string_to_json_value_escape_quotes())
    TEST(test_string_to_json_value_escape_backslash())
    TEST(test_string_to_json_value_escape_control_characters())
    TEST(test_empty_array_to_json_array())
    TEST(test_array_to_json_array())
    TEST(test_array_to_json_array_escape())
    return success ? 0 : 1;
}
