cmake_minimum_required(VERSION 3.16)
project(vvcoreExample)


set(LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../core/lib")

add_executable(example main.cpp)
target_include_directories(example PRIVATE "${LIB_DIR}")

add_compile_options("$<$<C_COMPILER_ID:MSVC>:/utf-8>")
add_compile_options("$<$<CXX_COMPILER_ID:MSVC>:/utf-8>")

file(GLOB coreLib "${LIB_DIR}/*.dll" "${LIB_DIR}/*.dylib" "${LIB_DIR}/*.so")
install(TARGETS example
	RUNTIME DESTINATION "${CMAKE_CURRENT_SOURCE_DIR}/result")
install(FILES ${coreLib}
	DESTINATION ${CMAKE_CURRENT_SOURCE_DIR}/result)
