cmake_minimum_required(VERSION 3.16)

project(SimpleTTS)

add_executable(simple_tts simple_tts.cpp)
set_property(TARGET simple_tts PROPERTY CXX_STANDARD 11)

file(GLOB ONNXRUNTIME_SHARED_LIB ./libonnxruntime.so.* ./libonnxruntime.*.dylib)
target_link_directories(simple_tts PRIVATE ./)

file(GLOB CORE_LIB ./libcore.so.* ./libcore.*.dylib)
target_link_libraries(simple_tts core)
