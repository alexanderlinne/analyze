#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

const char *safe_sprintf(const char *format, ...);

const char *linkname_of(int file_descriptor);

const char *bytes_to_hex_string(const void *bytes, size_t count);

size_t microseconds_since_epoch();

const char *string_to_json_value(const char *str);
const char *json_value_array_to_json_list(const char * const *arr);
const char *string_array_to_json_list(const char * const *arr);

#ifdef __cplusplus
} // extern "C"
#endif
