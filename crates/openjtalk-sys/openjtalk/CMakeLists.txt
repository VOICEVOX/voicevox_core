cmake_minimum_required(VERSION 3.16)

include(FetchContent)
FetchContent_Declare(
  openjtalk
  GIT_REPOSITORY https://github.com/VOICEVOX/open_jtalk.git
  GIT_TAG 1.11
  SOURCE_SUBDIR src)
FetchContent_MakeAvailable(openjtalk)

install(
  TARGETS openjtalk
  ARCHIVE DESTINATION lib
  LIBRARY DESTINATION lib
  RUNTIME DESTINATION lib)
