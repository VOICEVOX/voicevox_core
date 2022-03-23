#include "acoustic_feature_extractor.h"

namespace voicevox::core::engine {
long OjtPhoneme::phoneme_id() const {
  if (phoneme.empty()) return (long)-1;
  return (long)phoneme_map().at(phoneme);
}

std::vector<OjtPhoneme> OjtPhoneme::convert(std::vector<OjtPhoneme> phonemes) {
  if (phonemes[0].phoneme.find("sil") != std::string::npos) {
    phonemes[0].phoneme = OjtPhoneme::space_phoneme();
  }
  if (phonemes.back().phoneme.find("sil") != std::string::npos) {
    phonemes.back().phoneme = OjtPhoneme::space_phoneme();
  }
  return phonemes;
}
}  // namespace voicevox::core::engine
