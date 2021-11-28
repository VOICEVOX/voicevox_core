cmake_minimum_required(VERSION 3.16)
project(VoiceVoxCore)

# TODO: build options
set(ENABLE_CUDA OFF CACHE BOOL "use CUDA")

# TODO: download onnxruntime
set(ONNXRUNTIME_DIR "../core/onnxruntime-win-x64-1.9.0" CACHE PATH "Path to ONNX Runtime")

add_subdirectory(core)
