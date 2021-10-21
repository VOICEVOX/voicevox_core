cmake_minimum_required(VERSION 3.16)

project(ONNXCore)
set(ENABLE_CUDA OFF CACHE BOOL "use CUDA")

set(ONNXRUNTIME_DIR "" CACHE PATH "Path to ONNX Runtime")
if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
	set(CMAKE_INSTALL_PREFIX "${CMAKE_CURRENT_SOURCE_DIR}" CACHE PATH "Path to install" FORCE)
endif(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
message("core will be installed to: ${CMAKE_INSTALL_PREFIX}")

get_filename_component(ONNXRUNTIME_DIR ${ONNXRUNTIME_DIR} ABSOLUTE)
if(EXISTS ${ONNXRUNTIME_DIR}/VERSION_NUMBER)
	file(READ "${ONNXRUNTIME_DIR}/VERSION_NUMBER" ONNXRUNTIME_VERSION)
	message("ONNX Runtime Version: ${ONNXRUNTIME_VERSION}")
else()
	message(FATAL_ERROR "Unable to find ONNX Runtime. Use option -DONNXRUNTIME_DIR=...")
endif()
file(GLOB ONNXRUNTIME_LIBS
	"${ONNXRUNTIME_DIR}/lib/*.dll"
	"${ONNXRUNTIME_DIR}/lib/*.lib"
	"${ONNXRUNTIME_DIR}/lib/*.so"
	"${ONNXRUNTIME_DIR}/lib/*.so.*")

add_library(core SHARED core.cpp)
set_property(TARGET core PROPERTY CXX_STANDARD 17)
set_property(TARGET core PROPERTY POSITION_INDEPENDENT_CODE ON) # fPIC
set_property(TARGET core PROPERTY INSTALL_RPATH "$ORIGIN") # make enable to search onnxruntime.so
target_compile_options(core PRIVATE
	$<$<CXX_COMPILER_ID:MSVC>: /W4 /O2 /utf-8>
	$<$<CXX_COMPILER_ID:GNU>: -Wall -Wextra -O2>
)
target_include_directories(core
	PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/../
	PRIVATE ${ONNXRUNTIME_DIR}/include)
target_link_directories(core PUBLIC ${ONNXRUNTIME_DIR}/lib)
target_link_libraries(core PUBLIC onnxruntime)

# std::filesystem in GCC < 9.0 (https://gitlab.kitware.com/cmake/cmake/-/issues/17834)
target_link_libraries(core PRIVATE $<$<AND:$<CXX_COMPILER_ID:GNU>,$<VERSION_LESS:$<CXX_COMPILER_VERSION>,9.0>>:stdc++fs>)

if(ENABLE_CUDA)
	target_compile_definitions(core PRIVATE USE_CUDA)
endif(ENABLE_CUDA)

install(TARGETS core
	ARCHIVE DESTINATION lib
	LIBRARY DESTINATION lib
	RUNTIME DESTINATION lib)
install(FILES ${ONNXRUNTIME_LIBS} ${CMAKE_CURRENT_SOURCE_DIR}/../core.h
	DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
