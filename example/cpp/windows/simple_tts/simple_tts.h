#pragma once
#include <iostream>
#include "core.h"

std::string GetOpenJTalkDict();
std::wstring GetWaveFileName();
std::wstring GetExePath();
std::wstring GetExeDirectory();
void OutErrorMessage(VoicevoxResultCode messageCode);
std::string wide_to_utf8_cppapi(std::wstring const& src);
std::wstring utf8_to_wide_cppapi(std::string const& src);