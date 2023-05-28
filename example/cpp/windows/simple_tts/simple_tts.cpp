﻿// simple_tts.cpp : このファイルには 'main' 関数が含まれています。プログラム実行の開始と終了がそこで行われます。
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
#include <fstream>

#include "voicevox_core.h"

#define OPENJTALK_DICT_NAME L"open_jtalk_dic_utf_8-1.11"

int main() {
  std::wcout.imbue(std::locale(""));
  std::wcin.imbue(std::locale(""));

  std::wcout << L"生成する音声の文字列を入力" << std::endl;
  std::wcout << L">";
  std::wstring speak_words;
  std::wcin >> speak_words;

  std::wcout << L"coreの初期化中" << std::endl;
  VoicevoxInitializeOptions  initializeOptions = voicevox_default_initialize_options;
  std::string dict = GetOpenJTalkDict();
  initializeOptions.load_all_models = true;

  OpenJtalkRc* open_jtalk;
  auto result = voicevox_open_jtalk_rc_new(dict.c_str(),&open_jtalk);
  if (result != VoicevoxResultCode::VOICEVOX_RESULT_OK) {
    OutErrorMessage(result);
    return 0;
  }
  VoicevoxSynthesizer* synthesizer;
  result = voicevox_synthesizer_new_with_initialize(open_jtalk,initializeOptions,&synthesizer);
  if (result != VoicevoxResultCode::VOICEVOX_RESULT_OK) {
    OutErrorMessage(result);
    return 0;
  }
  voicevox_open_jtalk_rc_delete(open_jtalk);

  std::wcout << L"音声生成中" << std::endl;
  int32_t speaker_id = 0;
  uintptr_t output_binary_size = 0;
  uint8_t* output_wav = nullptr;
  VoicevoxTtsOptions ttsOptions = voicevox_default_tts_options;

  result = voicevox_synthesizer_tts(synthesizer,wide_to_utf8_cppapi(speak_words).c_str(), speaker_id, ttsOptions, &output_binary_size, &output_wav);
  if (result != VoicevoxResultCode::VOICEVOX_RESULT_OK) {
    OutErrorMessage(result);
    return 0;
  }

  {
    //音声ファイルの保存
    std::ofstream out_stream(GetWaveFileName().c_str(), std::ios::binary);
    out_stream.write(reinterpret_cast<const char*>(output_wav), output_binary_size);
    std::wcout << GetWaveFileName() << L" に保存しました。" << std::endl;
  }  //ここでファイルが閉じられる

  std::wcout << L"音声再生中" << std::endl;
  PlaySound((LPCTSTR)output_wav, nullptr, SND_MEMORY);

  std::wcout << L"音声データの開放" << std::endl;
  voicevox_wav_free(output_wav);

  voicevox_synthesizer_delete(synthesizer);
}

/// <summary>
/// OpenJTalk辞書のパスを取得します。
/// </summary>
/// <returns>OpenJTalk辞書のパス</returns>
std::string GetOpenJTalkDict() {
  wchar_t buff[MAX_PATH] = {0};
  PathCchCombine(buff, MAX_PATH, GetExeDirectory().c_str(), OPENJTALK_DICT_NAME);
  std::string retVal = wide_to_utf8_cppapi(buff);
  return retVal;
}

/// <summary>
/// 音声ファイル名を取得します。
/// </summary>
/// <returns>音声ファイルのフルパス</returns>
std::wstring GetWaveFileName() {
  wchar_t buff[MAX_PATH] = {0};
  PathCchCombine(buff, MAX_PATH, GetExeDirectory().c_str(), L"speech.wav");
  return std::wstring(buff);
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
/// コンソール画面にエラーメッセージを出力します。
/// </summary>
/// <param name="messageCode">メッセージコード</param>
void OutErrorMessage(VoicevoxResultCode messageCode) {
  const char* utf8Str = voicevox_error_result_to_message(messageCode);
  std::wstring wideStr = utf8_to_wide_cppapi(utf8Str);
  std::wcout << wideStr << std::endl;
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

/// <summary>
/// UTF8をワイド文字に変換します。
/// </summary>
/// <param name="src">UTF8文字列</param>
/// <returns>ワイド文字列</returns>
/// <remarks>
/// https://nekko1119.hatenablog.com/entry/2017/01/02/054629 から引用
/// </remarks>
std::wstring utf8_to_wide_cppapi(std::string const& src) {
  std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> converter;
  return converter.from_bytes(src);
}
