#define _GNU_SOURCE

#include "libc.h"

#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>

int(*real_access)(const char *, int) = NULL;
int(*real_close)(int) = NULL;
int(*real_execve)(const char *, char *const[], char *const[]) = NULL;
int(*real_execv)(const char *, char *const[]) = NULL;
int(*real_execvp)(const char *, char *const[]) = NULL;
int(*real_execvpe)(const char *, char *const[], const char *const[]) = NULL;
int(*real_fexecve)(int, char *const[], const char *const[]) = NULL;
void(*real_exit)(int) = NULL;
void(*real__exit)(int) = NULL;
void(*real__Exit)(int) = NULL;
int(*real_mkdir)(const char *, mode_t) = NULL;
int(*real_open)(const char *, int, mode_t) = NULL;
int(*real_sprintf)(char *str, const char *format, ...) = NULL;
int(*real_vsprintf)(char *str, const char *format, va_list ap) = NULL;
int(*real_vsnprintf)(char *str, size_t size, const char *format, va_list ap) = NULL;
ssize_t(*real_write)(int, const void *, size_t) = NULL;

static void *libc_function(const char *function_name)
{
    void *function = dlsym(RTLD_NEXT, function_name);
    if (!function) {
        printf("Cannot find symbol \'%s\'!\n", function_name);
        abort();
    }
    return function;
}

void init_libc_function_pointers() {
    real_access = libc_function("access");
    real_close = libc_function("close");
    real_execve = libc_function("execve");
    real_execv = libc_function("execv");
    real_execvp = libc_function("execvp");
    real_execvpe = libc_function("execvpe");
    real_fexecve = libc_function("fexecve");
    real_exit = libc_function("exit");
    real__exit = libc_function("_exit");
    real__Exit = libc_function("_Exit");
    real_mkdir = libc_function("mkdir");
    real_open = libc_function("open");
    real_sprintf = libc_function("sprintf");
    real_vsprintf = libc_function("vsprintf");
    real_vsnprintf = libc_function("vsnprintf");
    real_write = libc_function("write");
}
