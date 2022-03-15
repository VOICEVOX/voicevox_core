#include <cstdlib>
#include <memory>
#include <stdexcept>
#include <vector>

#include "core.h"
#include "engine/model.h"
#include "engine/kana_parser.h"
#include "engine/synthesis_engine.h"

using namespace voicevox::core::engine;

static SynthesisEngine engine;

VoicevoxResultCode voicevox_load_openjtalk_dict(const char *dict_path) {
  // TODO: error handling
  engine.load_openjtalk_dict(dict_path);
  return VOICEVOX_RESULT_SUCCEED;
}

VoicevoxResultCode voicevox_tts(const char *text, int64_t speaker_id, int *output_binary_size, uint8_t **output_wav) {
  if (!engine.is_openjtalk_dict_loaded()) {
    return VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT;
  }

  std::vector<AccentPhraseModel> accent_phrases = engine.create_accent_phrases(std::string(text), &speaker_id);
  const AudioQueryModel audio_query = {
      accent_phrases, 1.0f, 0.0f, 1.0f, 1.0f, 0.1f, 0.1f, engine.default_sampling_rate, false, "",
  };

  const auto wav = engine.synthesis_wave_format(audio_query, &speaker_id, output_binary_size);
  auto *wav_heap = new uint8_t[*output_binary_size];
  std::copy(wav.begin(), wav.end(), wav_heap);
  *output_wav = wav_heap;
  return VOICEVOX_RESULT_SUCCEED;
}

VoicevoxResultCode voicevox_tts_from_aquestalk_notation(const char *text, int64_t speaker_id, int *output_binary_size,
                                                        uint8_t **output_wav) {
  std::vector<AccentPhraseModel> accent_phrases = parse_kana(std::string(text));
  accent_phrases = engine.replace_mora_data(accent_phrases, &speaker_id);
  const AudioQueryModel audio_query = {
      accent_phrases, 1.0f, 0.0f, 1.0f, 1.0f, 0.1f, 0.1f, engine.default_sampling_rate, false, "",
  };

  const auto wav = engine.synthesis_wave_format(audio_query, &speaker_id, output_binary_size);
  auto *wav_heap = new uint8_t[*output_binary_size];
  std::copy(wav.begin(), wav.end(), wav_heap);
  *output_wav = wav_heap;
  return VOICEVOX_RESULT_SUCCEED;
}

void voicevox_wav_free(uint8_t *wav) { delete wav; }

const char *voicevox_error_result_to_message(VoicevoxResultCode result_code) {
  switch (result_code) {
    case VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT:
      return "Call voicevox_load_openjtalk_dict() first.";

    default:
      throw std::runtime_error("Unexpected error result code.");
  }
}
