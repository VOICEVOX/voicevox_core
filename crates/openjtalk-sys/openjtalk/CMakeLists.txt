cmake_minimum_required(VERSION 3.16)
project(OpenjtalkSys)

include(FetchContent)
FetchContent_Declare(
  openjtalk
  GIT_REPOSITORY https://github.com/VOICEVOX/open_jtalk.git
  GIT_TAG 1.11
  SOURCE_SUBDIR src)
FetchContent_MakeAvailable(openjtalk)

add_library(OpenjtalkSys src/openjtalk.cpp)
set_property(TARGET OpenjtalkSys PROPERTY CXX_STANDARD 17)
set_property(TARGET OpenjtalkSys PROPERTY POSITION_INDEPENDENT_CODE ON)
target_compile_options(
  OpenjtalkSys
  PRIVATE $<$<CXX_COMPILER_ID:MSVC>:
          /W4
          /O2
          /utf-8>
          $<$<CXX_COMPILER_ID:GNU>:
          -Wall
          -Wextra
          -O2>)
set_property(TARGET openjtalk PROPERTY CXX_STANDARD 17)
set_property(TARGET openjtalk PROPERTY POSITION_INDEPENDENT_CODE ON)
target_compile_options(
  openjtalk
  PRIVATE $<$<CXX_COMPILER_ID:MSVC>:
          /W4
          /O2
          /utf-8>
          $<$<CXX_COMPILER_ID:GNU>:
          -Wall
          -Wextra
          -O2>)

target_link_libraries(OpenjtalkSys PUBLIC openjtalk)

install(
  TARGETS openjtalk
  ARCHIVE DESTINATION lib
  LIBRARY DESTINATION lib
  RUNTIME DESTINATION lib)
