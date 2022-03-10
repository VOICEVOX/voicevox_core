#pragma once

#include <map>
#include <string>
#include <vector>

#include "openjtalk.h"

namespace voicevox::core::engine {
std::string string_feature_by_regex(std::string pattern, std::string label);

class Phoneme {
 public:
  std::map<std::string, std::string> contexts;
  std::string label;

  Phoneme(const std::map<std::string, std::string> contexts, const std::string label) {
    this->contexts = contexts;
    this->label = label;
  }

  static Phoneme *from_label(const std::string &label);

  std::string phoneme();
  bool is_pause();
};

class Mora {
 public:
  Phoneme *consonant = nullptr;
  Phoneme *vowel;

  Mora(Phoneme *vowel) { this->vowel = vowel; }

  Mora(Phoneme *consonant, Phoneme *vowel) {
    this->consonant = consonant;
    this->vowel = vowel;
  }

  void set_context(const std::string &key, const std::string &value) const;
  std::vector<Phoneme *> phonemes();
  std::vector<std::string> labels();
};

class AccentPhrase {
 public:
  std::vector<Mora *> moras;
  unsigned int accent;
  bool is_interrogative;

  AccentPhrase(std::vector<Mora *> moras, unsigned int accent, bool is_interrogative) {
    this->moras = moras;
    this->accent = accent;
    this->is_interrogative = is_interrogative;
  }

  static AccentPhrase *from_phonemes(std::vector<Phoneme *> phonemes);
  void set_context(std::string key, std::string value);
  std::vector<Phoneme *> phonemes();
  std::vector<std::string> labels();
  AccentPhrase *merge(AccentPhrase *accent_phrase);
};

class BreathGroup {
 public:
  std::vector<AccentPhrase *> accent_phrases;

  BreathGroup(std::vector<AccentPhrase *> accent_phrases) { this->accent_phrases = accent_phrases; }

  static BreathGroup *from_phonemes(std::vector<Phoneme *> phonemes);
  void set_context(std::string key, std::string value);
  std::vector<Phoneme *> phonemes();
  std::vector<std::string> labels();
};

class Utterance {
 public:
  std::vector<BreathGroup *> breath_groups;
  std::vector<Phoneme *> pauses;

  Utterance(std::vector<BreathGroup *> breath_groups, std::vector<Phoneme *> pauses) {
    this->breath_groups = breath_groups;
    this->pauses = pauses;
  }

  static Utterance from_phonemes(const std::vector<Phoneme *> &phonemes);
  void set_context(const std::string &key, const std::string &value);
  std::vector<Phoneme *> phonemes();
  std::vector<std::string> labels();
};

Utterance extract_full_context_label(OpenJTalk *openjtalk, std::string text);
}
