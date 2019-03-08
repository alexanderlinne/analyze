cmake_minimum_required(VERSION 3.1)

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

add_library(preload_common OBJECT
    file_report.c
    libc.c
    report.c
    util.c
)

add_library(preload
    $<TARGET_OBJECTS:preload_common>
    access.c
    exec.c
    main.c
)

target_link_libraries(preload PRIVATE
    dl
)

add_executable(test_libc_preload
    $<TARGET_OBJECTS:preload_common>
    main.test.c
)

target_link_libraries(test_libc_preload
    dl
)

add_test(test_libc_preload test_libc_preload)
