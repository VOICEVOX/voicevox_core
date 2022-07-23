cmake_minimum_required(VERSION 3.16)

project(SimpleTTS)

execute_process(COMMAND cargo build --release WORKING_DIRECTORY ../../../)
add_executable(simple_tts simple_tts.cpp)
target_include_directories(simple_tts PRIVATE ../../../target)
file(
  GLOB
  ONNXRUNTIME_SHARED_LIB_DIR
  ../../../target/release/build/onnxruntime-sys-*/out/onnxruntime_*/onnxruntime-linux-*/lib
)

file(GLOB ONNXRUNTIME_SHARED_LIB
     ${ONNXRUNTIME_SHARED_LIB_DIR}/*onnxruntime*.so*
     ${ONNXRUNTIME_SHARED_LIB_DjR}/*onnxruntime*.dylib)

set(CORE_LIB_DIR ../../../target/release)
file(GLOB CORE_LIB ${CORE_LIB_DIR}/*core*)
target_link_directories(simple_tts PRIVATE ${CORE_LIB_DIR}
                        ${ONNXRUNTIME_SHARED_LIB_DIR})
target_link_libraries(simple_tts core)

add_custom_command(
  TARGET simple_tts
  POST_BUILD
  COMMAND ${CMAKE_COMMAND} -E copy ${ONNXRUNTIME_SHARED_LIB} ${CORE_LIB}
          $<TARGET_FILE_DIR:simple_tts>)
