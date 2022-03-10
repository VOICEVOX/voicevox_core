#include <cstdlib>
#include <exception>
#include <vector>

#include "core.h"
#include "engine/model.h"
#include "engine/openjtalk.h"
#include "engine/synthesis_engine.h"

using namespace voicevox::core::engine;

static OpenJTalk *openjtalk = nullptr;
static SynthesisEngine *engine = nullptr;

VoicevoxResultCode voicevox_initialize_openjtalk(const char *dict_path) {
  // TODO: error handling
  openjtalk = new OpenJTalk(dict_path);
  return VoicevoxResultSucceed;
}

VoicevoxResultCode voicevox_tts(const char *text, int64_t speaker_id, int binary_size, uint8_t **wav_out) {
  if (openjtalk == nullptr) {
    return VoicevoxResultNotInitializedOpenJTalkErr;
  }
  if (engine == nullptr) {
    engine = new SynthesisEngine(openjtalk);
  }

  std::vector<AccentPhraseModel> accent_phrases = engine->create_accent_phrases(std::string(text), &speaker_id);
  const AudioQueryModel audio_query = {
      accent_phrases, 1.0f, 0.0f, 1.0f, 1.0f, 0.1f, 0.1f, engine->default_sampling_rate, false, "",
  };

  const auto wav = engine->synthesis_wave_format(audio_query, &speaker_id, &binary_size);
  auto *wav_heap = new uint8_t[binary_size];
  std::copy(wav.begin(), wav.end(), wav_heap);
  *wav_out = wav_heap;
  return VoicevoxResultSucceed;
}

void voicevox_wav_free(uint8_t *wav) { delete wav; }

const char *voicevox_error_result_to_message(VoicevoxResultCode result_code) {
  switch (result_code) {
    case VoicevoxResultNotInitializedOpenJTalkErr:
      return "Call initialize_openjtalk() first.";

    default:
      throw std::runtime_error("Unexpected error result code.");
  }
}
