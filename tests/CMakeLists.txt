cmake_minimum_required(VERSION 3.16)

project(VoiceVoxCoreTest)

include(FetchContent)
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v3.0.0-preview5)
FetchContent_MakeAvailable(Catch2)
file(GLOB_RECURSE unit_test_files "unit_tests/*.cpp")
add_executable(unit_test ${unit_test_files})
target_include_directories(unit_test PRIVATE ${Catch2_SOURCE_DIR}/src)
target_include_directories(unit_test PRIVATE ${CORE_DIR}/src)
target_link_libraries(unit_test PRIVATE Catch2::Catch2WithMain)
target_link_libraries(unit_test PRIVATE core)
list(APPEND CMAKE_MODULE_PATH ${Catch2_SOURCE_DIR}/extras)
include(Catch)
include(CTest)
catch_discover_tests(unit_test)
