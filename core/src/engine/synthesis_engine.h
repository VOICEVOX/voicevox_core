#pragma once

#include <sstream>
#include <string>
#include <vector>

#include "../core.h"
#include "acoustic_feature_extractor.h"
#include "model.h"
#include "openjtalk.h"

static std::vector<std::string> unvoiced_mora_phoneme_list = {"A", "I", "U", "E", "O", "cl", "pau"};

static std::vector<std::string> mora_phoneme_list = {"a", "i", "u", "e", "o",  "N",  "A",
                                                     "I", "U", "E", "O", "cl", "pau"};

std::vector<MoraModel> to_flatten_moras(std::vector<AccentPhraseModel> accent_phrases);
std::vector<OjtPhoneme> to_phoneme_data_list(std::vector<std::string> phoneme_str_list);
void split_mora(std::vector<OjtPhoneme> phoneme_list, std::vector<OjtPhoneme> &consonant_phoneme_list,
                std::vector<OjtPhoneme> &vowel_phoneme_list, std::vector<long> &vowel_indexes);
std::vector<AccentPhraseModel> adjust_interrogative_accent_phrases(std::vector<AccentPhraseModel> accent_phrases);
std::vector<MoraModel> adjust_interrogative_moras(AccentPhraseModel accent_phrase);
MoraModel make_interrogative_mora(MoraModel last_mora);

class SynthesisEngine {
 public:
  const unsigned int default_sampling_rate = 24000;

  SynthesisEngine(OpenJTalk *openjtalk) { m_openjtalk = openjtalk; }

  std::vector<AccentPhraseModel> create_accent_phrases(std::string text, int64_t *speaker_id);
  std::vector<AccentPhraseModel> replace_mora_data(std::vector<AccentPhraseModel> accent_phrases, int64_t *speaker_id);
  std::vector<AccentPhraseModel> replace_phoneme_length(std::vector<AccentPhraseModel> accent_phrases,
                                                        int64_t *speaker_id);
  std::vector<AccentPhraseModel> replace_mora_pitch(std::vector<AccentPhraseModel> accent_phrases, int64_t *speaker_id);
  std::vector<float> synthesis(AudioQueryModel query, int64_t *speaker_id, bool enable_interrogative_upspeak = true);
  std::vector<uint8_t> synthesis_wave_format(AudioQueryModel query, int64_t *speaker_id, int *binary_size,
                                    bool enable_interrogative_upspeak = true);

 private:
  OpenJTalk *m_openjtalk;

  void initial_process(std::vector<AccentPhraseModel> &accent_phrases, std::vector<MoraModel> &flatten_moras,
                       std::vector<std::string> &phoneme_str_list, std::vector<OjtPhoneme> &phoneme_data_list);
  void create_one_accent_list(std::vector<int64_t> &accent_list, AccentPhraseModel accent_phrase, int point);
};
