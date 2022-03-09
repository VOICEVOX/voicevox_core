cmake_minimum_required(VERSION 3.16)
project(VoiceVoxCore)

# TODO: download onnxruntime
set(ONNXRUNTIME_DIR "${CMAKE_CURRENT_SOURCE_DIR}/onnxruntime" CACHE PATH "Path to ONNX Runtime")
set(MODEL_DIR "${CMAKE_CURRENT_SOURCE_DIR}/model" CACHE PATH "Path to model")

add_subdirectory(core)
