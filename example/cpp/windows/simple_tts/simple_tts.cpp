// simple_tts.cpp : このファイルには 'main' 関数が含まれています。プログラム実行の開始と終了がそこで行われます。
//

#include <iostream>
#include <Windows.h>
#include <shlwapi.h>
#include <string.h>
#include <pathcch.h>
#include <codecvt>
#include <vector>
#include <array>
#include "..\..\..\core\src\core.h"
#include "simple_tts.h"

#define OPENJTALK_DICT_NAME L"open_jtalk_dic_utf_8-1.11"

int main()
{
    std::wcout.imbue(std::locale(""));
    std::wcin.imbue(std::locale(""));

    std::wcout << L"生成する音声の文字列を入力" << std::endl;
    std::wcout << L">";
    std::wstring speak_words;
    std::wcin >> speak_words;
    std::wcout << speak_words << std::endl;
    
    //coreの初期化
    initialize(false);

    VoicevoxResultCode result;
    result = voicevox_load_openjtalk_dict(GetOpenJTalkDict().c_str());
    if (result != VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED) {
        std::cout << voicevox_error_result_to_message(result) << std::endl;
        return 0;
    }


}

//OpenJTalk辞書のパスを取得します。
std::string GetOpenJTalkDict() { 
    wchar_t buff[MAX_PATH] = {0};
    PathCchCombine(buff, MAX_PATH, GetExeDirectory().c_str(), OPENJTALK_DICT_NAME);
    std::string retVal = wide_to_multi_capi(buff);
    //std::string retVal((char*)buff);

    //ConvU32ToU8((char32_t*)buff, retVal);
    return retVal;
}

//自分自身のあるパスを取得する
std::wstring GetExePath() {
    wchar_t buff[MAX_PATH] = {0};
    GetModuleFileName(nullptr, buff, MAX_PATH);
    return std::wstring(buff);
}

//自分自身のあるディレクトリを取得する
std::wstring GetExeDirectory() { 
    wchar_t buff[MAX_PATH] = {0};
    wcscpy_s(buff, MAX_PATH, GetExePath().c_str());
    //フルパスからファイル名の削除
    PathRemoveFileSpec(buff);
    return std::wstring(buff);
}

//ワイド文字をShift_JISに変換します。
//https://nekko1119.hatenablog.com/entry/2017/01/02/054629 から引用
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

// プログラムの実行: Ctrl + F5 または [デバッグ] > [デバッグなしで開始] メニュー
// プログラムのデバッグ: F5 または [デバッグ] > [デバッグの開始] メニュー

// 作業を開始するためのヒント: 
//    1. ソリューション エクスプローラー ウィンドウを使用してファイルを追加/管理します 
//   2. チーム エクスプローラー ウィンドウを使用してソース管理に接続します
//   3. 出力ウィンドウを使用して、ビルド出力とその他のメッセージを表示します
//   4. エラー一覧ウィンドウを使用してエラーを表示します
//   5. [プロジェクト] > [新しい項目の追加] と移動して新しいコード ファイルを作成するか、[プロジェクト] > [既存の項目の追加] と移動して既存のコード ファイルをプロジェクトに追加します
//   6. 後ほどこのプロジェクトを再び開く場合、[ファイル] > [開く] > [プロジェクト] と移動して .sln ファイルを選択します
