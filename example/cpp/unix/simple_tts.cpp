#include <fstream>
#include <iostream>
#include <string>

#include "voicevox_core/voicevox_core.h"

#define OUTPUT_WAV_NAME "audio.wav"

int main(int argc, char *argv[]) {
  if (argc != 2) {
    std::cout << "使い方: ./simple_tts <文章>" << std::endl;
    return 0;
  }

  std::string open_jtalk_dict_path("voicevox_core/open_jtalk_dic_utf_8-1.11");
  std::string text(argv[1]);

  std::cout << "coreの初期化中..." << std::endl;

  auto initialize_options = voicevox_make_default_initialize_options();
  OpenJtalkRc* open_jtalk;
  auto result = voicevox_open_jtalk_rc_new(open_jtalk_dict_path.c_str(),&open_jtalk);
  if (result != VOICEVOX_RESULT_OK){
    std::cerr << voicevox_error_result_to_message(result) << std::endl;
    return 1;
  }
  VoicevoxSynthesizer* synthesizer;
  result = voicevox_synthesizer_new_with_initialize(open_jtalk,initialize_options,&synthesizer);
  if (result != VOICEVOX_RESULT_OK) {
    std::cerr << voicevox_error_result_to_message(result) << std::endl;
    return 1;
  }
  voicevox_open_jtalk_rc_delete(open_jtalk);

  VoicevoxVoiceModel* model;
  result = voicevox_voice_model_new_from_path("..\\..\\..\\model\\sample.vvm");
  if (result != VoicevoxResultCode::VOICEVOX_RESULT_OK) {
    std::cerr << voicevox_error_result_to_message(result) << std::endl;
    return 0;
  }
  result = voicevox_synthesizer_load_voice_model(synthesizer, model);
  if (result != VoicevoxResultCode::VOICEVOX_RESULT_OK) {
    std::cerr << voicevox_error_result_to_message(result) << std::endl;
    return 0;
  }
  voicevox_voice_model_delete(model);

  std::cout << "音声生成中..." << std::endl;

  int64_t style_id = 0;
  size_t output_wav_size = 0;
  uint8_t *output_wav = nullptr;

  result = voicevox_synthesizer_tts(synthesizer,text.c_str(), style_id,
                             voicevox_make_default_tts_options(),
                             &output_wav_size, &output_wav);
  if (result != VOICEVOX_RESULT_OK) {
    std::cerr << voicevox_error_result_to_message(result) << std::endl;
    return 1;
  }

  std::cout << "音声ファイル保存中..." << std::endl;

  std::ofstream wav_file(OUTPUT_WAV_NAME, std::ios::binary);
  wav_file.write(reinterpret_cast<const char *>(output_wav), output_wav_size);
  voicevox_wav_free(output_wav);

  std::cout << "音声ファイル保存完了 (" << OUTPUT_WAV_NAME << ")" << std::endl;

  voicevox_synthesizer_delete(synthesizer);

  return 0;
}
