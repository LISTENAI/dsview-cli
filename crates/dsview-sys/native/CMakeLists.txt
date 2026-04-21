cmake_minimum_required(VERSION 3.16)
project(dsview_runtime C CXX)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

if(NOT DEFINED DSVIEW_REPO_ROOT)
    message(FATAL_ERROR "DSVIEW_REPO_ROOT must point at the repository root")
endif()

file(TO_CMAKE_PATH "${DSVIEW_REPO_ROOT}" DSVIEW_REPO_ROOT_NORMALIZED)
file(TO_CMAKE_PATH "${DSVIEW_REPO_ROOT_NORMALIZED}/DSView" DSVIEW_ROOT)
file(TO_CMAKE_PATH "${DSVIEW_ROOT}/libsigrok4DSL" LIBSIGROK_ROOT)
file(TO_CMAKE_PATH "${DSVIEW_ROOT}/common" COMMON_ROOT)
file(TO_CMAKE_PATH "${DSVIEW_REPO_ROOT_NORMALIZED}/crates/dsview-sys/compat" DSVIEW_SYS_COMPAT_ROOT)
file(TO_CMAKE_PATH "${CMAKE_CURRENT_LIST_DIR}/windows" DSVIEW_NATIVE_WINDOWS_ROOT)

find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB REQUIRED glib-2.0)
pkg_check_modules(LIBUSB REQUIRED libusb-1.0)
pkg_check_modules(FFTW REQUIRED fftw3)

set(LIBUSB_NORMALIZED_INCLUDE_DIRS ${LIBUSB_INCLUDE_DIRS})
foreach(LIBUSB_INCLUDE_DIR IN LISTS LIBUSB_INCLUDE_DIRS)
    get_filename_component(LIBUSB_INCLUDE_BASENAME "${LIBUSB_INCLUDE_DIR}" NAME)
    if(LIBUSB_INCLUDE_BASENAME STREQUAL "libusb-1.0")
        get_filename_component(LIBUSB_INCLUDE_PARENT "${LIBUSB_INCLUDE_DIR}" DIRECTORY)
        list(APPEND LIBUSB_NORMALIZED_INCLUDE_DIRS "${LIBUSB_INCLUDE_PARENT}")
    endif()
endforeach()
list(REMOVE_DUPLICATES LIBUSB_NORMALIZED_INCLUDE_DIRS)

find_package(Threads REQUIRED)

set(DSVIEW_RUNTIME_SOURCES
    ${COMMON_ROOT}/log/xlog.c
    ${LIBSIGROK_ROOT}/version.c
    ${LIBSIGROK_ROOT}/strutil.c
    ${LIBSIGROK_ROOT}/std.c
    ${LIBSIGROK_ROOT}/session.c
    ${LIBSIGROK_ROOT}/log.c
    ${LIBSIGROK_ROOT}/hwdriver.c
    ${LIBSIGROK_ROOT}/error.c
    ${LIBSIGROK_ROOT}/backend.c
    ${LIBSIGROK_ROOT}/input/in_vcd.c
    ${LIBSIGROK_ROOT}/hardware/DSL/dslogic.c
    ${LIBSIGROK_ROOT}/hardware/common/usb.c
    ${LIBSIGROK_ROOT}/hardware/common/ezusb.c
    ${LIBSIGROK_ROOT}/trigger.c
    ${LIBSIGROK_ROOT}/dsdevice.c
    ${LIBSIGROK_ROOT}/hardware/DSL/dscope.c
    ${LIBSIGROK_ROOT}/hardware/DSL/command.c
    ${LIBSIGROK_ROOT}/hardware/DSL/dsl.c
    ${LIBSIGROK_ROOT}/lib_main.c
)

if(MSVC)
    list(APPEND DSVIEW_RUNTIME_SOURCES
        ${DSVIEW_NATIVE_WINDOWS_ROOT}/input_minimal.c
        ${DSVIEW_NATIVE_WINDOWS_ROOT}/output_minimal.c
        ${DSVIEW_NATIVE_WINDOWS_ROOT}/session_stubs.c
    )
else()
    find_package(ZLIB REQUIRED)
    list(APPEND DSVIEW_RUNTIME_SOURCES
        ${COMMON_ROOT}/minizip/ioapi.c
        ${COMMON_ROOT}/minizip/unzip.c
        ${COMMON_ROOT}/minizip/zip.c
        ${LIBSIGROK_ROOT}/session_driver.c
        ${LIBSIGROK_ROOT}/output/output.c
        ${LIBSIGROK_ROOT}/input/input.c
        ${LIBSIGROK_ROOT}/hardware/demo/demo.c
        ${LIBSIGROK_ROOT}/input/in_binary.c
        ${LIBSIGROK_ROOT}/input/in_wav.c
        ${LIBSIGROK_ROOT}/output/csv.c
        ${LIBSIGROK_ROOT}/output/gnuplot.c
        ${LIBSIGROK_ROOT}/output/srzip.c
        ${LIBSIGROK_ROOT}/output/vcd.c
    )
endif()

add_library(dsview_runtime SHARED ${DSVIEW_RUNTIME_SOURCES})

target_include_directories(dsview_runtime PRIVATE
    ${DSVIEW_SYS_COMPAT_ROOT}
    ${DSVIEW_ROOT}
    ${LIBSIGROK_ROOT}
    ${COMMON_ROOT}
    ${GLIB_INCLUDE_DIRS}
    ${LIBUSB_NORMALIZED_INCLUDE_DIRS}
    ${FFTW_INCLUDE_DIRS}
)

target_compile_definitions(dsview_runtime PRIVATE
    HAVE_LIBUSB_1_0=1
    HAVE_DSL_DEVICE=1
)

if(NOT MSVC)
    target_compile_definitions(dsview_runtime PRIVATE HAVE_LA_DEMO=1)
endif()

if(MSVC)
    target_compile_options(dsview_runtime PRIVATE
        /W3
        /utf-8
        /FI${DSVIEW_SYS_COMPAT_ROOT}/msvc_preinclude.h
        /wd4100  # unreferenced formal parameter
        /wd4101  # unreferenced local variable
        /wd4189  # local variable initialized but not referenced
    )
else()
    target_compile_options(dsview_runtime PRIVATE
        -Wall
        -Wextra
        -Wno-unused-parameter
        -Wno-unused-function
        -Wno-sign-compare
        -Wno-missing-field-initializers
        -Wno-discarded-qualifiers
    )
endif()

target_link_libraries(dsview_runtime PRIVATE
    Threads::Threads
    ${GLIB_LIBRARIES}
    ${LIBUSB_LIBRARIES}
    ${FFTW_LIBRARIES}
)

if(NOT MSVC)
    target_link_libraries(dsview_runtime PRIVATE ZLIB::ZLIB)
endif()

target_link_directories(dsview_runtime PRIVATE
    ${GLIB_LIBRARY_DIRS}
    ${LIBUSB_LIBRARY_DIRS}
    ${FFTW_LIBRARY_DIRS}
)

# Link math library on Unix-like systems
if(UNIX)
    target_link_libraries(dsview_runtime PRIVATE m)
endif()

if(MSVC)
    target_link_libraries(dsview_runtime PRIVATE ws2_32)
endif()

set_target_properties(dsview_runtime PROPERTIES
    OUTPUT_NAME dsview_runtime
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
)
