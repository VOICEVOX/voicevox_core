#pragma once

#include <map>
#include <optional>
#include <string>
#include <vector>

#include "openjtalk.h"

namespace voicevox::core::engine {
std::string string_feature_by_regex(std::string pattern, std::string label);

class Phoneme {
 public:
  std::map<std::string, std::string> contexts;
  std::string label;

  Phoneme(const std::map<std::string, std::string> contexts, const std::string label)
      : contexts(contexts), label(label) {}

  static Phoneme from_label(const std::string &label);

  std::string phoneme() const;
  bool is_pause() const;
};

class Mora {
 public:
  std::optional<Phoneme> consonant;
  Phoneme vowel;

  Mora(Phoneme vowel) : vowel(vowel) {}

  Mora(Phoneme consonant, Phoneme vowel) : consonant(consonant), vowel(vowel) {}

  void set_context(const std::string &key, const std::string &value);
  std::vector<Phoneme> phonemes() const;
  std::vector<std::string> labels();
};

class AccentPhrase {
 public:
  std::vector<Mora> moras;
  unsigned int accent;
  bool is_interrogative;

  AccentPhrase(std::vector<Mora> moras, unsigned int accent, bool is_interrogative)
      : moras(moras), accent(accent), is_interrogative(is_interrogative) {}

  static AccentPhrase from_phonemes(std::vector<Phoneme> phonemes);
  void set_context(std::string key, std::string value);
  std::vector<Phoneme> phonemes() const;
  std::vector<std::string> labels();
  AccentPhrase merge(AccentPhrase &accent_phrase);
};

class BreathGroup {
 public:
  std::vector<AccentPhrase> accent_phrases;

  BreathGroup(std::vector<AccentPhrase> accent_phrases) : accent_phrases(accent_phrases) {}

  static BreathGroup from_phonemes(std::vector<Phoneme> &phonemes);
  void set_context(std::string key, std::string value);
  std::vector<Phoneme> phonemes() const;
  std::vector<std::string> labels();
};

class Utterance {
 public:
  std::vector<BreathGroup> breath_groups;
  std::vector<Phoneme> pauses;

  Utterance(std::vector<BreathGroup> breath_groups, std::vector<Phoneme> pauses)
      : breath_groups(breath_groups), pauses(pauses) {}

  static Utterance from_phonemes(const std::vector<Phoneme> &phonemes);
  void set_context(const std::string &key, const std::string &value);
  std::vector<Phoneme> phonemes();
  std::vector<std::string> labels();
};

Utterance extract_full_context_label(OpenJTalk &openjtalk, std::string text);
}  // namespace voicevox::core::engine
