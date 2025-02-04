cmake_minimum_required(VERSION 3.16)

project(SimpleTTS)

add_executable(simple_tts simple_tts.cpp)
set_property(TARGET simple_tts PROPERTY CXX_STANDARD 17)

file(GLOB ONNXRUNTIME_SHARED_LIB ./libonnxruntime.so.* ./libonnxruntime.*.dylib) # TODO: なにこれ
target_link_directories(simple_tts PRIVATE ./voicevox_core/c_api/lib/)


file(GLOB CORE_LIB ./voicevox_core/c_api/lib/libvoicevox_core.so.* ./voicevox_core/c_api/lib/libvoicevox_core.*.dylib) # TODO: なにこれ
target_link_libraries(simple_tts voicevox_core)
