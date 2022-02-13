cmake_minimum_required(VERSION 3.16)
project(VoiceVoxCore)

# TODO: download onnxruntime
set(ONNXRUNTIME_DIR "${CMAKE_CURRENT_SOURCE_DIR}/onnxruntime" CACHE PATH "Path to ONNX Runtime")

option(DIRECTML "Enables building for DirectML" OFF)
set(DIRECTML_DIR "${CMAKE_CURRENT_SOURCE_DIR}/directml" CACHE PATH "Path to DirectML")
if(DIRECTML)
  # DirectML読み込み用にCPUのアーキテクチャを求める
  set(DML_ARCH)
  include(${CMAKE_CURRENT_SOURCE_DIR}/DetermineTargetArchitecture.cmake)
  determine_target_architecture(DML_ARCH)
  message("DML_ARCH - ${DML_ARCH}")
endif()

add_subdirectory(core)
