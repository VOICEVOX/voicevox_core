// simple_tts.cpp : このファイルには 'main' 関数が含まれています。プログラム実行の開始と終了がそこで行われます。
//

#include "simple_tts.h"

#include <Windows.h>
#include <pathcch.h>
#include <shlwapi.h>
#include <string.h>

#include <array>
#include <codecvt>
#include <iostream>
#include <vector>

#include "..\..\..\core\src\core.h"

#define OPENJTALK_DICT_NAME L"open_jtalk_dic_utf_8-1.11"

int main() {
  std::wcout.imbue(std::locale(""));
  std::wcin.imbue(std::locale(""));

  std::wcout << L"生成する音声の文字列を入力" << std::endl;
  std::wcout << L">";
  std::wstring speak_words;
  std::wcin >> speak_words;

  std::wcout << L"coreの初期化中" << std::endl;
  initialize(false);

  VoicevoxResultCode result = VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED;

  std::wcout << L"openjtalk辞書の読み込み" << std::endl;
  result = voicevox_load_openjtalk_dict(GetOpenJTalkDict().c_str());
  if (result != VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED) {
    std::cout << voicevox_error_result_to_message(result) << std::endl;
    return 0;
  }

  std::wcout << L"音声生成中" << std::endl;
  int64_t speaker_id = 0;
  int output_binary_size = 0;
  uint8_t* output_wav = nullptr;
  result = voicevox_tts(wide_to_utf8_cppapi(speak_words).c_str(), speaker_id, &output_binary_size, &output_wav);
  if (result != VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED) {
    std::cout << voicevox_error_result_to_message(result) << std::endl;
    return 0;
  }

  std::wcout << L"音声再生中" << std::endl;
  PlaySound((LPCTSTR)output_wav, nullptr, SND_MEMORY);

  std::wcout << L"音声データの開放" << std::endl;
  voicevox_wav_free(output_wav);

}

/// <summary>
/// OpenJTalk辞書のパスを取得します。
/// </summary>
/// <returns>OpenJTalk辞書のパス</returns>
std::string GetOpenJTalkDict() {
  wchar_t buff[MAX_PATH] = {0};
  PathCchCombine(buff, MAX_PATH, GetExeDirectory().c_str(), OPENJTALK_DICT_NAME);
  std::string retVal = wide_to_multi_capi(buff);
  return retVal;
}

/// <summary>
/// 自分自身のあるパスを取得する
/// </summary>
/// <returns>自分のexeのフルパス</returns>
std::wstring GetExePath() {
  wchar_t buff[MAX_PATH] = {0};
  GetModuleFileName(nullptr, buff, MAX_PATH);
  return std::wstring(buff);
}

/// <summary>
/// 自分自身のあるディレクトリを取得する
/// </summary>
/// <returns>自分のexeのあるディレクトリ</returns>
std::wstring GetExeDirectory() {
  wchar_t buff[MAX_PATH] = {0};
  wcscpy_s(buff, MAX_PATH, GetExePath().c_str());
  //フルパスからファイル名の削除
  PathRemoveFileSpec(buff);
  return std::wstring(buff);
}

/// <summary>
/// ワイド文字列をShift_JISに変換します。
/// </summary>
/// <param name="src">ワイド文字列</param>
/// <returns>Shift_JIS文字列</returns>
/// <remarks>
/// https://nekko1119.hatenablog.com/entry/2017/01/02/054629 から引用
/// </remarks>
std::string wide_to_multi_capi(std::wstring const& src) {
  std::size_t converted{};
  std::vector<char> dest(src.size() * sizeof(wchar_t) + 1, '\0');
  if (::_wcstombs_s_l(&converted, dest.data(), dest.size(), src.data(), _TRUNCATE, ::_create_locale(LC_ALL, "jpn")) !=
      0) {
    throw std::system_error{errno, std::system_category()};
  }
  dest.resize(std::char_traits<char>::length(dest.data()));
  dest.shrink_to_fit();
  return std::string(dest.begin(), dest.end());
}

/// <summary>
/// ワイド文字列をUTF8に変換します。
/// </summary>
/// <param name="src">ワイド文字列</param>
/// <returns>UTF8文字列</returns>
/// <remarks>
/// https://nekko1119.hatenablog.com/entry/2017/01/02/054629 から引用
/// </remarks>
std::string wide_to_utf8_cppapi(std::wstring const& src) {
  std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> converter;
  return converter.to_bytes(src);
}
