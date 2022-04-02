#pragma once
#include <iostream>

std::string GetOpenJTalkDict();
std::wstring GetExePath();
std::wstring GetExeDirectory();
std::string wide_to_multi_capi(std::wstring const& src);
std::string wide_to_utf8_cppapi(std::wstring const& src);