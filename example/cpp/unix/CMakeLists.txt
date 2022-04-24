cmake_minimum_required(VERSION 3.16)

project(SimpleTTS)

add_executable(simple_tts simple_tts.cpp)

find_library(
  CORE_LIB
  NAMES core libcore
  HINTS "${CMAKE_SOURCE_DIR}"
  NO_DEFAULT_PATH
)

message(STATUS "CORE_LIB: ${CORE_LIB}")
if(NOT CORE_LIB)
  message(FATAL_ERROR "Unable to find VOICEVOX core library")
endif()

target_link_libraries(simple_tts ${CORE_LIB})
set_property(TARGET simple_tts PROPERTY CXX_STANDARD 11)
