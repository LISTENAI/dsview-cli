cmake_minimum_required(VERSION 3.16)
project(dsview_runtime C CXX)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

if(NOT DEFINED DSVIEW_REPO_ROOT)
    message(FATAL_ERROR "DSVIEW_REPO_ROOT must point at the repository root")
endif()

set(DSVIEW_ROOT "${DSVIEW_REPO_ROOT}/DSView")
set(LIBSIGROK_ROOT "${DSVIEW_ROOT}/libsigrok4DSL")
set(COMMON_ROOT "${DSVIEW_ROOT}/common")
set(DSVIEW_SYS_COMPAT_ROOT "${DSVIEW_REPO_ROOT}/crates/dsview-sys/compat")

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

find_package(ZLIB REQUIRED)
find_package(Threads REQUIRED)

add_library(dsview_runtime SHARED
    ${COMMON_ROOT}/log/xlog.c
    ${COMMON_ROOT}/minizip/ioapi.c
    ${COMMON_ROOT}/minizip/unzip.c
    ${COMMON_ROOT}/minizip/zip.c
    ${LIBSIGROK_ROOT}/version.c
    ${LIBSIGROK_ROOT}/strutil.c
    ${LIBSIGROK_ROOT}/std.c
    ${LIBSIGROK_ROOT}/session_driver.c
    ${LIBSIGROK_ROOT}/session.c
    ${LIBSIGROK_ROOT}/log.c
    ${LIBSIGROK_ROOT}/hwdriver.c
    ${LIBSIGROK_ROOT}/error.c
    ${LIBSIGROK_ROOT}/backend.c
    ${LIBSIGROK_ROOT}/output/output.c
    ${LIBSIGROK_ROOT}/input/input.c
    ${LIBSIGROK_ROOT}/hardware/demo/demo.c
    ${LIBSIGROK_ROOT}/input/in_binary.c
    ${LIBSIGROK_ROOT}/input/in_vcd.c
    ${LIBSIGROK_ROOT}/input/in_wav.c
    ${LIBSIGROK_ROOT}/output/csv.c
    ${LIBSIGROK_ROOT}/output/gnuplot.c
    ${LIBSIGROK_ROOT}/output/srzip.c
    ${LIBSIGROK_ROOT}/output/vcd.c
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
    HAVE_LA_DEMO=1
)

if(MSVC)
    target_compile_options(dsview_runtime PRIVATE
        /W3
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
    ZLIB::ZLIB
    Threads::Threads
    ${GLIB_LIBRARIES}
    ${LIBUSB_LIBRARIES}
    ${FFTW_LIBRARIES}
)

target_link_directories(dsview_runtime PRIVATE
    ${GLIB_LIBRARY_DIRS}
    ${LIBUSB_LIBRARY_DIRS}
    ${FFTW_LIBRARY_DIRS}
)

# Link math library on Unix-like systems
if(UNIX)
    target_link_libraries(dsview_runtime PRIVATE m)
endif()

set_target_properties(dsview_runtime PROPERTIES
    OUTPUT_NAME dsview_runtime
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
)
