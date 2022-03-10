#include <cstdlib>
#include <exception>
#include <vector>

#include "core.h"
#include "engine/model.h"
#include "engine/openjtalk.h"
#include "engine/synthesis_engine.h"

#define NOT_INITIALIZED_OPENJTALK_ERR "Call initialize_openjtalk() first."

using namespace voicevox::core::engine;

static OpenJTalk *openjtalk = nullptr;
static SynthesisEngine *engine = nullptr;

bool initialize_openjtalk(const char *dict_path) {
  // TODO: error handling
  openjtalk = new OpenJTalk(dict_path);
  return true;
}


uint8_t *voicevox_tts(const char *text, int64_t *speaker_id, int *binary_size) {
  if (openjtalk == nullptr) {
    throw std::runtime_error(NOT_INITIALIZED_OPENJTALK_ERR);
  }
  if (engine == nullptr) {
    engine = new SynthesisEngine(openjtalk);
  }

  std::vector<AccentPhraseModel> accent_phrases = engine->create_accent_phrases(std::string(text), speaker_id);
  const AudioQueryModel audio_query = {
    accent_phrases,
    1.0f,
    0.0f,
    1.0f,
    1.0f,
    0.1f,
    0.1f,
    engine->default_sampling_rate,
    false,
    "",
  };

  const auto wav = engine->synthesis_wave_format(audio_query, speaker_id, binary_size);
  auto* wav_heap = new uint8_t[*binary_size];
  std::copy(wav.begin(),wav.end(),wav_heap);
  return wav_heap;
}

void voicevox_wav_free(uint8_t *wav) {
  delete wav;
}
