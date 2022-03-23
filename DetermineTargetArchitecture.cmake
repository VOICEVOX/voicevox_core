# https://github.com/civetweb/civetweb/blob/master/cmake/DetermineTargetArchitecture.cmake
#
# Copyright (c) 2013-2021 The CivetWeb developers (https://github.com/civetweb/civetweb/blob/master/CREDITS.md)
# Copyright (c) 2004-2013 Sergey Lyubka
# Copyright (c) 2013 No Face Press, LLC (Thomas Davis)
# Copyright (c) 2013 F-Secure Corporation
#
# Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
# 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
# 
# - Determines the target architecture of the compilation
#
# This function checks the architecture that will be built by the compiler
# and sets a variable to the architecture
#
#  determine_target_architecture(<OUTPUT_VAR>)
#
# - Example
#
# include(DetermineTargetArchitecture)
# determine_target_architecture(PROJECT_NAME_ARCHITECTURE)

if(__determine_target_architecture)
  return()
endif()
set(__determine_target_architecture INCLUDED)

function(determine_target_architecture FLAG)
  if (MSVC)
    if("${MSVC_C_ARCHITECTURE_ID}" STREQUAL "X86")
      set(ARCH "x86")
    elseif("${MSVC_C_ARCHITECTURE_ID}" STREQUAL "x64")
      set(ARCH "x64")
    elseif("${MSVC_C_ARCHITECTURE_ID}" STREQUAL "ARM64")
      set(ARCH "arm64")
    elseif("${MSVC_C_ARCHITECTURE_ID}" MATCHES  "ARM*")
      set(ARCH "arm")
    else()
      message(FATAL_ERROR "Failed to determine the MSVC target architecture: ${MSVC_C_ARCHITECTURE_ID}")
    endif()
  else()
    execute_process(
      COMMAND ${CMAKE_C_COMPILER} -dumpmachine
      RESULT_VARIABLE RESULT
      OUTPUT_VARIABLE ARCH
      ERROR_QUIET
    )
    if (RESULT)
      message(FATAL_ERROR "Failed to determine target architecture triplet: ${RESULT}")
    endif()
    string(REGEX MATCH "([^-]+).*" ARCH_MATCH ${ARCH})
    if (NOT CMAKE_MATCH_1 OR NOT ARCH_MATCH)
      message(FATAL_ERROR "Failed to match the target architecture triplet: ${ARCH}")
    endif()
    set(ARCH ${CMAKE_MATCH_1})
  endif()
  message(STATUS "Target architecture - ${ARCH}")
  set(${FLAG} ${ARCH} PARENT_SCOPE)
endfunction()
