cmake_minimum_required(VERSION 3.16)
project(OpenJtalkSys)

include(FetchContent)
FetchContent_Declare(
  openjtalk
  GIT_REPOSITORY https://github.com/VOICEVOX/open_jtalk.git
  GIT_TAG 1.11
  SOURCE_SUBDIR src)
FetchContent_MakeAvailable(openjtalk)
add_library(OpenJtalkSys src/openjtalk.cpp)
target_link_libraries(OpenJtalkSys PUBLIC openjtalk)

install(
  TARGETS OpenJtalkSys
  ARCHIVE DESTINATION lib
  LIBRARY DESTINATION lib
  RUNTIME DESTINATION lib)
