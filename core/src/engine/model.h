#pragma once

#include <optional>
#include <string>
#include <vector>

namespace voicevox::core::engine {
struct MoraModel {
  std::string text;
  std::optional<std::string> consonant;
  std::optional<float> consonant_length;
  std::string vowel;
  float vowel_length;
  float pitch;
};

struct AccentPhraseModel {
  std::vector<MoraModel> moras;
  unsigned int accent;
  std::optional<MoraModel> pause_mora;
  bool is_interrogative;
};

struct AudioQueryModel {
  std::vector<AccentPhraseModel> accent_phrases;
  float speed_scale;
  float pitch_scale;
  float intonation_scale;
  float volume_scale;
  float pre_phoneme_length;
  float post_phoneme_length;
  unsigned int output_sampling_rate;
  bool output_stereo;
  std::string kana;
};
}  // namespace voicevox::core::engine
