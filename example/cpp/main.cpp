#include <algorithm>
#include <exception>
#include <fstream>
#include <iostream>
#include <locale>
#include <stdexcept>
#include <string>
#include <vector>

#include "param.h"

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
  const auto result_code = voicevox_tts(msg.c_str(), speaker_id, &binary_size, &wav);
  if (result_code != VOICEVOX_RESULT_SUCCEED) {
    std::cout << "error:" << voicevox_error_result_to_message(result_code) << std::endl;
    return 0;
  }
  std::cout << result_code << std::endl;
  // 後続の処理を実装

  ofs.write((char *)wav, binary_size);
  ofs.flush();
  ofs.close();

  // 使い終わったら開放する
  voicevox_wav_free(wav);
  std::cout << binary_size << std::endl;

  FreeLibrary(handler);
  return 0;
}