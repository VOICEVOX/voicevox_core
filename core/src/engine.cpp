#include <cstdlib>
#include <exception>
#include <vector>

#include "core.h"
#include "engine/model.h"
#include "engine/openjtalk.h"
#include "engine/synthesis_engine.h"


#define NOT_INITIALIZED_OPENJTALK_ERR "Call initialize_openjtalk() first."

static OpenJTalk *openjtalk = nullptr;
static SynthesisEngine *engine = nullptr;
// free関数が必要かも
static const char *wave_binary = nullptr;

bool initialize_openjtalk(const char *dict_path) {
  // TODO: error handling
  openjtalk = new OpenJTalk(dict_path);
  return true;
}


const char *tts(const char *text, int64_t *speaker_id, int *binary_size) {
  if (openjtalk == nullptr) {
    throw std::runtime_error(NOT_INITIALIZED_OPENJTALK_ERR);
  }
  if (engine == nullptr) {
    engine = new SynthesisEngine(openjtalk);
  }

  std::vector<AccentPhraseModel> accent_phrases = engine->create_accent_phrases(std::string(text), speaker_id);
  accent_phrases = engine->replace_mora_data(accent_phrases, speaker_id);
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

  const char *wav = engine->synthesis_wave_format(audio_query, speaker_id, binary_size);
  wave_binary = wav;
  return wave_binary;
}
