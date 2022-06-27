cmake_minimum_required(VERSION 3.16)
project(VoiceVoxCore)

# TODO: download onnxruntime
set(ONNXRUNTIME_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/onnxruntime"
    CACHE PATH "Path to ONNX Runtime")
set(MODEL_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/model"
    CACHE PATH "Path to model")
set(CORE_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/core"
    CACHE PATH "Path to core")
set(DEPENDENT_DLLS
    ""
    CACHE INTERNAL "Dependent DLLs of core.dll")

set(CMAKE_POSITION_INDEPENDENT_CODE ON)

option(DIRECTML "Enables building for DirectML" OFF)
set(DIRECTML_DIR "${CMAKE_CURRENT_SOURCE_DIR}/directml" CACHE PATH "Path to ONNX Runtime")
if(DIRECTML)
  # DirectML読み込み用にCPUのアーキテクチャを求める
  set(DML_ARCH)
  include(${CMAKE_CURRENT_SOURCE_DIR}/DetermineTargetArchitecture.cmake)
  determine_target_architecture(DML_ARCH)
endif()

option(ARMHF "Enables building for ARMHF" OFF)
if(ARMHF)
  # https://github.com/VOICEVOX/voicevox_core/issues/132
  add_link_options("-Xlinker" "--long-plt")
endif()

add_subdirectory(core)
add_subdirectory(open_jtalk/src)

if(BUILD_TEST)
  enable_testing()
  add_subdirectory(tests)
endif()
