#include <fstream>
#include <iostream>
#include <string>

#include "core.h"

#define OUTPUT_WAV_NAME "audio.wav"

int main(int argc, char *argv[]) {
  if (argc != 2) {
    std::cout << "使い方: ./simple_tts <文章>" << std::endl;
    return 0;
  }

  std::string open_jtalk_dict_path("voicevox_core/open_jtalk_dic_utf_8-1.11");
  std::string text(argv[1]);

  std::cout << "coreの初期化中..." << std::endl;

  if (!initialize(false, 0, true)) {
    std::cout << "coreの初期化に失敗しました" << std::endl;
    return 1;
  }

  VoicevoxResultCode result;

  std::cout << "openjtalk辞書の読み込み中..." << std::endl;

  result = voicevox_load_openjtalk_dict(open_jtalk_dict_path.c_str());
  if (result != VOICEVOX_RESULT_SUCCEED) {
    std::cout << voicevox_error_result_to_message(result) << std::endl;
    return 1;
  }

  std::cout << "音声生成中..." << std::endl;

  int64_t speaker_id = 0;
  int output_binary_size = 0;
  uint8_t *output_wav = nullptr;

  result =
      voicevox_tts(text.c_str(), speaker_id, &output_binary_size, &output_wav);
  if (result != VOICEVOX_RESULT_SUCCEED) {
    std::cout << voicevox_error_result_to_message(result) << std::endl;
    return 1;
  }

  std::cout << "音声ファイル保存中..." << std::endl;

  std::ofstream wav_file(OUTPUT_WAV_NAME, std::ios::binary);
  wav_file.write(reinterpret_cast<const char *>(output_wav),
                 output_binary_size);
  voicevox_wav_free(output_wav);

  std::cout << "音声ファイル保存完了 (" << OUTPUT_WAV_NAME << ")" << std::endl;

  return 0;
}
