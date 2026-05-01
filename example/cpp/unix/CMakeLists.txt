cmake_minimum_required(VERSION 3.16)

project(VoicevoxCoreExamples)

add_executable(talk talk.cpp)
add_executable(song song.cpp)
set_property(TARGET talk song PROPERTY CXX_STANDARD 17)

file(GLOB ONNXRUNTIME_SHARED_LIB ./libonnxruntime.so.* ./libonnxruntime.*.dylib) # TODO: なにこれ
target_link_directories(talk PRIVATE ./voicevox_core/c_api/lib/)
target_link_directories(song PRIVATE ./voicevox_core/c_api/lib/)


file(GLOB CORE_LIB ./voicevox_core/c_api/lib/libvoicevox_core.so.* ./voicevox_core/c_api/lib/libvoicevox_core.*.dylib) # TODO: なにこれ
target_link_libraries(talk voicevox_core)
target_link_libraries(song voicevox_core)
