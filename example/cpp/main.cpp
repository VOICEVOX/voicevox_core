#include <algorithm>
#include <exception>
#include <fstream>
#include <iostream>
#include <locale>
#include <stdexcept>
#include <string>
#include <vector>

#define CORE_PATH "./core.dll"
#define MODEL_DIR "../../../model"
#define OPENJTALK_DIR "../dic"

#ifdef _WIN32
#include <windows.h>
#else
#include <dlfcn.h>
#define LoadLibrary(path) dlopen(path, RTLD_LAZY)
#define GetProcAddress(handler, func_name) dlsym(handler, func_name);
#define FreeLibrary(handler) dlclose(handler);
typedef void *HMODULE;
typedef void *FARPROC;
#endif

typedef enum {
  // 成功
  VOICEVOX_RESULT_SUCCEED = 0,
  // OpenJTalk初期化に失敗した
  VOICEVOX_RESULT_NOT_INITIALIZE_OPEN_JTALK_ERR = 1,
} VoicevoxResultCode;

using INIT = bool (*)(const char *root_dir_path, bool use_gpu, int cpu_num_threads);
using FIN = void (*)();
using INIT_OJT = VoicevoxResultCode (*)(const char *dict_path);
using TTS = VoicevoxResultCode (*)(const char *text, int64_t speaker_id, int *output_binary_size, uint8_t **output_wav);
using FREE = void (*)(uint8_t *wav);
using ERR2MSG = const char *(*)(VoicevoxResultCode result_code);

std::string SjistoUTF8(std::string srcSjis) {
  // Unicodeへ変換後の文字列長を得る
  int lenghtUnicode = MultiByteToWideChar(CP_THREAD_ACP, 0, srcSjis.c_str(), srcSjis.size() + 1, NULL, 0);

  //必要な分だけUnicode文字列のバッファを確保
  wchar_t *bufUnicode = new wchar_t[lenghtUnicode];

  // ShiftJISからUnicodeへ変換
  MultiByteToWideChar(CP_THREAD_ACP, 0, srcSjis.c_str(), srcSjis.size() + 1, bufUnicode, lenghtUnicode);

  // UTF8へ変換後の文字列長を得る
  int lengthUTF8 = WideCharToMultiByte(CP_UTF8, 0, bufUnicode, -1, NULL, 0, NULL, NULL);

  //必要な分だけUTF8文字列のバッファを確保
  char *bufUTF8 = new char[lengthUTF8];

  // UnicodeからUTF8へ変換
  WideCharToMultiByte(CP_UTF8, 0, bufUnicode, lenghtUnicode + 1, bufUTF8, lengthUTF8, NULL, NULL);

  std::string strUTF8(bufUTF8);

  delete bufUnicode;
  delete bufUTF8;

  return strUTF8;
}

int main(int argc, char *argv[]) {
  std::locale::global(std::locale(""));

  // get args
  std::cout << "argc: " << argc << std::endl;
  std::cout << "arg1: text:" << argv[1] << std::endl;

  // load dlls
  std::cout << "loading core..." << std::endl;
  HMODULE handler = LoadLibrary(CORE_PATH);
  if (handler == nullptr) {
#ifndef _WIN32
    std::cout << dlerror() << std::endl;
#endif
    std::cout << "failed to load core library" << std::endl;
    throw std::runtime_error("failed to load core library");
  }

  INIT initialize = (INIT)GetProcAddress(handler, "initialize");
  FIN finalize = (FIN)GetProcAddress(handler, "finalize");
  INIT_OJT initialize_ojt = (INIT_OJT)GetProcAddress(handler, "voicevox_initialize_openjtalk");
  TTS voicevox_tts = (TTS)GetProcAddress(handler, "voicevox_tts");
  FREE voicevox_wav_free = (FREE)GetProcAddress(handler, "voicevox_wav_free");
  ERR2MSG voicevox_error_result_to_message = (ERR2MSG)GetProcAddress(handler, "voicevox_error_result_to_message");

  if (initialize == nullptr || initialize_ojt == nullptr || voicevox_tts == nullptr || voicevox_wav_free == nullptr ||
      voicevox_error_result_to_message == nullptr) {
    throw std::runtime_error("to load library is succeeded, but can't found needed functions");
  }
  std::cout << "loaded!" << std::endl;

  // Initialize core
  std::cout << "initializing..." << std::endl;
  if (!initialize(MODEL_DIR, false, 0)) {
    std::cout << "failed to initialize core library" << std::endl;
    throw std::runtime_error("failed to initialize core library");
  }

  // Initialize openjtalk
  std::cout << "initialized!" << std::endl << "initializing openjtalk..." << std::endl;
  initialize_ojt(OPENJTALK_DIR);
  std::cout << "initialized openjtalk!" << std::endl;

  // Create wav file
  std::ofstream ofs("./test.wav", std::ios::binary);

  // Generate voice
  std::string msg = argv[1];
  std::cout << msg << std::endl;

  uint8_t *wav;
  int64_t speaker_id = 1;
  int binary_size;
  const auto result_code = voicevox_tts(SjistoUTF8(msg).c_str(), speaker_id, &binary_size, &wav);
  if (result_code != VOICEVOX_RESULT_SUCCEED) {
    std::cout << "error:" << voicevox_error_result_to_message(result_code) << std::endl;
    return 0;
  }

  std::cout << binary_size << std::endl;

  ofs.write((char *)wav, binary_size);
  ofs.flush();
  ofs.close();

  // 使い終わったら開放する
  voicevox_wav_free(wav);
  finalize();
  FreeLibrary(handler);
  return 0;
}