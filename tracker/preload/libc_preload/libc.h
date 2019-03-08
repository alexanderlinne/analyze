#pragma once

#include <unistd.h>
#include <stdarg.h>
#include <stddef.h>
#include <sys/stat.h>

extern int(*real_access)(const char *, int);
extern int(*real_close)(int);
extern int(*real_execve)(const char *, char *const[], char *const[]);
extern int(*real_execv)(const char *, char *const[]);
extern int(*real_execvp)(const char *, char *const[]);
extern int(*real_execvpe)(const char *, char *const[], const char *const[]);
extern int(*real_fexecve)(int, char *const[], const char *const[]);
extern void(*real_exit)(int);
extern void(*real__exit)(int);
extern void(*real__Exit)(int);
extern int(*real_mkdir)(const char *, mode_t);
extern int(*real_open)(const char *, int, mode_t);
extern int(*real_sprintf)(char *str, const char *format, ...);
extern int(*real_vsprintf)(char *str, const char *format, va_list ap);
extern int(*real_vsnprintf)(char *str, size_t size, const char *format, va_list ap);
extern ssize_t(*real_write)(int, const void *, size_t);

#ifdef __cplusplus
extern "C" {
#endif

void init_libc_function_pointers();

#ifdef __cplusplus
} // extern "C"
#endif
