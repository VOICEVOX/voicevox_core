#include <cstdlib>
#include <exception>
#include <memory>
#include <vector>

#include "core.h"
#include "engine/model.h"
#include "engine/openjtalk.h"
#include "engine/synthesis_engine.h"

using namespace voicevox::core::engine;

// TODO:SynthesisEngineにopenjtalkを持たせるためshared_ptrにしているが、やめたい
static std::shared_ptr<OpenJTalk> openjtalk;
static std::unique_ptr<SynthesisEngine> engine;

VoicevoxResultCode voicevox_initialize_openjtalk(const char *dict_path) {
  // TODO: error handling
  openjtalk = std::make_shared<OpenJTalk>(dict_path);
  return VOICEVOX_RESULT_SUCCEED;
}

VoicevoxResultCode voicevox_tts(const char *text, int64_t speaker_id, int *output_binary_size, uint8_t **output_wav) {
  if (!openjtalk) {
    return VOICEVOX_RESULT_NOT_INITIALIZE_OPEN_JTALK_ERR;
  }
  if (!engine) {
    engine = std::make_unique<SynthesisEngine>(openjtalk);
  }

  std::vector<AccentPhraseModel> accent_phrases = engine->create_accent_phrases(std::string(text), &speaker_id);
  const AudioQueryModel audio_query = {
      accent_phrases, 1.0f, 0.0f, 1.0f, 1.0f, 0.1f, 0.1f, engine->default_sampling_rate, false, "",
  };

  const auto wav = engine->synthesis_wave_format(audio_query, &speaker_id, output_binary_size);
  auto *wav_heap = new uint8_t[*output_binary_size];
  std::copy(wav.begin(), wav.end(), wav_heap);
  *output_wav = wav_heap;
  return VOICEVOX_RESULT_SUCCEED;
}

void voicevox_wav_free(uint8_t *wav) { delete wav; }

const char *voicevox_error_result_to_message(VoicevoxResultCode result_code) {
  switch (result_code) {
    case VOICEVOX_RESULT_NOT_INITIALIZE_OPEN_JTALK_ERR:
      return "Call initialize_openjtalk() first.";

    default:
      throw std::runtime_error("Unexpected error result code.");
  }
}
