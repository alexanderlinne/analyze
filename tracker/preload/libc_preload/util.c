#include "util.h"

#include <assert.h>
#include <dlfcn.h>
#include <unistd.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>

#include "libc.h"

const size_t INITIAL_BUFFER_SIZE = 1024;
const size_t MAX_BUFFER_SIZE = 1048576; // max. 1MB

const char *safe_sprintf(const char *format, ...) {
    va_list args;

    va_start(args, format);
    int result = real_vsnprintf(NULL, 0, format, args);
    va_end(args);
    if (result < 0) {
        abort();
    }

    char *buffer = malloc(result + 1);
    va_start(args, format);
    result = real_vsprintf(buffer, format, args);
    if (result < 0) {
        abort();
    }
    return buffer;
}

bool call_readlink(const char *path, char *linkname, size_t max)
{
    const int result = readlink(path, linkname, max);
    if (result < 0) {
        abort();
    }
    if (result >= max) {
        return false;
    }
    linkname[result] = '\0';
    return true;
}

const char *read_linkname_of(const int file_descriptor)
{
    const char *path = safe_sprintf("/proc/self/fd/%d", file_descriptor);
    size_t buffer_size = INITIAL_BUFFER_SIZE;
    char *linkname = NULL;
    do {
        linkname = realloc(linkname, buffer_size + 1);
        if (call_readlink(path, linkname, buffer_size + 1)) {
            free((void *)path);
            return linkname;
        } else {
            buffer_size *= 2;
            if (buffer_size > MAX_BUFFER_SIZE) {
                abort();
            }
        }
    } while (true);
}

const char *linkname_of(const int file_descriptor)
{
    char *buffer = malloc(7);
    if (file_descriptor <= 2) {
        buffer = malloc(7);
    }
    switch(file_descriptor) {
    case 0:
        return strcpy(buffer, "stdin");
    case 1:
        return strcpy(buffer, "stdout");
    case 2:
        return strcpy(buffer, "stderr");
    default:
        return read_linkname_of(file_descriptor);
    }
}

const char *bytes_to_hex_string(const void *bytes, size_t count)
{
    char *result = malloc(2 * count + 1);
    for (int i = 0; i < count; ++i) {
        real_sprintf(&result[2 * i], "%02x", ((unsigned char *)bytes)[i]);
    }
    return result;
}

size_t microseconds_since_epoch()
{
    struct timeval tp;
    gettimeofday(&tp, NULL);
    return tp.tv_sec * 1000000 + tp.tv_usec;
}

const char *string_to_json_value(const char *str)
{
    size_t input_length = strlen(str);
    size_t escape_count = 0;
    for (size_t i = 0; i < input_length; ++i) {
        if (str[i] == '"' || str[i] == '\\') {
            escape_count++;
        }
        if (!isprint(str[i])) {
            escape_count += 5;
        }
    }

    char *result = malloc(input_length + escape_count + 3);
    size_t write_position = 0;
    result[write_position++] = '"';
    for (size_t i = 0; i < input_length; ++i) {
        if (str[i] == '"' || str[i] == '\\') {
            result[write_position++] = '\\';
        }
        if (!isprint(str[i])) {
            sprintf(&result[write_position], "\\u%04X", str[i]);
            write_position += 6;
        } else {
            result[write_position++] = str[i];
        }
    }
    result[write_position++] = '"';
    result[write_position++] = '\0';
    assert(write_position == input_length + escape_count + 3);
    return result;
}

const char *json_value_array_to_json_list(const char * const *arr) {
    size_t count = 0;
    for (size_t i = 0; arr[i] != NULL; ++i) {
        count++;
    }

    size_t result_length = 0;
    for (size_t i = 0; arr[i] != NULL; ++i) {
        result_length += strlen(arr[i]);
    }

    // list seperators (count - 1)
    // + '[' + ']' + '\0'
    result_length += (count == 0 ? 0 : count - 1) + 3;

    char *result = malloc(result_length);

    result[0] = '[';
    size_t write_position = 1;
    for (size_t i = 0; arr[i] != NULL; ++i) {
        size_t length = strlen(arr[i]);
        memcpy(&result[write_position], arr[i], length);
        write_position += length;
        if (arr[i + 1]) {
            result[write_position++] = ',';
        }
    }
    result[write_position++] = ']';
    result[write_position++] = '\0';

    return result;
}

const char *string_array_to_json_list(const char * const *arr)
{
    size_t count = 0;
    for (size_t i = 0; arr[i] != NULL; ++i) {
        count++;
    }

    const char **json_values = malloc((count + 1) * sizeof(char *));
    for (size_t i = 0; arr[i] != NULL; ++i) {
        json_values[i] = string_to_json_value(arr[i]);
    }
    json_values[count] = NULL;

    const char *result = json_value_array_to_json_list(json_values);

    for (size_t i = 0; json_values[i] != NULL; ++i) {
        free((void *)json_values[i]);
    }
    free((void *)json_values);

    return result;
}
