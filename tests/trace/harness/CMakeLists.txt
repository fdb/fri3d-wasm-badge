# Trace harness for deterministic integration tests

add_executable(trace_harness
    trace_harness.c
    jsmn.c
)

target_include_directories(trace_harness PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_SOURCE_DIR}/src/runtime
)

target_link_libraries(trace_harness PRIVATE
    fri3d_runtime
)

if(APPLE)
    target_compile_definitions(trace_harness PRIVATE BH_PLATFORM_DARWIN)
elseif(UNIX)
    target_compile_definitions(trace_harness PRIVATE BH_PLATFORM_LINUX)
endif()

set_target_properties(trace_harness PROPERTIES
    C_STANDARD 11
    C_STANDARD_REQUIRED YES
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin
)
