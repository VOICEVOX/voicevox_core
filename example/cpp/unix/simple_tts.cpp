#include <fstream>
#include <iostream>
#include <string>

#include "voicevox_core.h"

#define OUTPUT_WAV_NAME "audio.wav"

int main(int argc, char *argv[]) {
  if (argc != 2) {
    std::cout << "使い方: ./simple_tts <文章>" << std::endl;
    return 0;
  }

  std::string open_jtalk_dict_path("open_jtalk_dic_utf_8-1.11");
  std::string text(argv[1]);

  std::cout << "coreの初期化中..." << std::endl;

  auto initialize_options = voicevox_make_default_initialize_options();
  initialize_options.load_all_models = true;
  initialize_options.open_jtalk_dict_dir = open_jtalk_dict_path.c_str();
  if (voicevox_initialize(initialize_options) != VOICEVOX_RESULT_SUCCEED) {
    std::cout << "coreの初期化に失敗しました" << std::endl;
    return 1;
  }

  std::cout << "音声生成中..." << std::endl;

  int64_t speaker_id = 0;
  size_t output_wav_size = 0;
  uint8_t *output_wav = nullptr;

  auto result = voicevox_tts(text.c_str(), speaker_id,
                             voicevox_make_default_tts_options(),
                             &output_wav_size, &output_wav);
  if (result != VOICEVOX_RESULT_SUCCEED) {
    std::cout << voicevox_error_result_to_message(result) << std::endl;
    return 1;
  }

  std::cout << "音声ファイル保存中..." << std::endl;

  std::ofstream wav_file(OUTPUT_WAV_NAME, std::ios::binary);
  wav_file.write(reinterpret_cast<const char *>(output_wav), output_wav_size);
  voicevox_wav_free(output_wav);

  std::cout << "音声ファイル保存完了 (" << OUTPUT_WAV_NAME << ")" << std::endl;

  return 0;
}
