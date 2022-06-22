cmake_minimum_required(VERSION 3.16)
project(OpenjtalkSys)

include(FetchContent)
FetchContent_Declare(
  openjtalk
  GIT_REPOSITORY https://github.com/VOICEVOX/open_jtalk.git
  GIT_TAG 1.11)
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

target_include_directories(
  OpenjtalkSys
  PRIVATE ${openjtalk_SOURCE_DIR}/src/jpcommon
  PRIVATE ${openjtalk_SOURCE_DIR}/src/mecab/src
  PRIVATE ${openjtalk_SOURCE_DIR}/src/mecab2njd
  PRIVATE ${openjtalk_SOURCE_DIR}/src/mecab-naist-jdic
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd2jpcommon
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd_set_accent_phrase
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd_set_accent_type
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd_set_digit
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd_set_long_vowel
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd_set_pronunciation
  PRIVATE ${openjtalk_SOURCE_DIR}/src/njd_set_unvoiced_vowel
  PRIVATE ${openjtalk_SOURCE_DIR}/src/text2mecab)
target_link_libraries(OpenjtalkSys PUBLIC openjtalk)

install(
  TARGETS OpenjtalkSys
  ARCHIVE DESTINATION lib
  LIBRARY DESTINATION lib
  RUNTIME DESTINATION lib)
install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/src/openjtalk.hpp
        DESTINATION ${CMAKE_INSTALL_PREFIX}/include)
# FetchContent_MakeAvailable(openjtalk)
#
# add_library(OpenjtalkSys src/openjtalk.cpp)
#
# set(CMAKE_MACOSX_RPATH 1) set_property(TARGET OpenjtalkSys PROPERTY
# CXX_STANDARD 17) set_property(TARGET OpenjtalkSys PROPERTY
# POSITION_INDEPENDENT_CODE ON)
#
# target_compile_options( OpenjtalkSys PRIVATE $<$<CXX_COMPILER_ID:MSVC>: /W4
# /O2 /utf-8> $<$<CXX_COMPILER_ID:GNU>: -Wall -Wextra -O2>)
#
# target_include_directories( OpenjtalkSys PUBLIC
# ${openjtalk_SOURCE_DIR}/src/jpcommon PUBLIC
# ${openjtalk_SOURCE_DIR}/src/mecab/src PUBLIC
# ${openjtalk_SOURCE_DIR}/src/mecab2njd PUBLIC
# ${openjtalk_SOURCE_DIR}/src/mecab-naist-jdic PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd2jpcommon PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd_set_accent_phrase PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd_set_accent_type PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd_set_digit PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd_set_long_vowel PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd_set_pronunciation PUBLIC
# ${openjtalk_SOURCE_DIR}/src/njd_set_unvoiced_vowel PUBLIC
# ${openjtalk_SOURCE_DIR}/src/text2mecab)
#
# target_link_libraries(OpenjtalkSys INTERFACE openjtalk)
#
# install( TARGETS OpenjtalkSys ARCHIVE DESTINATION lib LIBRARY DESTINATION lib
# RUNTIME DESTINATION lib)
