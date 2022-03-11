#include "full_context_label.h"

#include <algorithm>
#include <iostream>
#include <iterator>
#include <regex>
#include <stdexcept>

namespace voicevox::core::engine {
std::string string_feature_by_regex(std::string pattern, std::string label) {
  std::regex re(pattern);
  std::smatch match;
  if (std::regex_search(label, match, re)) {
    return match[1].str();
  } else {
    throw std::runtime_error("label is broken");
  }
}

Phoneme *Phoneme::from_label(const std::string &label) {
  std::map<std::string, std::string> contexts;
  contexts["p3"] = string_feature_by_regex(R"(\-(.*?)\+)", label);
  contexts["a2"] = string_feature_by_regex(R"(\+(\d+|xx)\+)", label);
  contexts["a3"] = string_feature_by_regex(R"(\+(\d+|xx)/B\:)", label);
  contexts["f1"] = string_feature_by_regex(R"(/F:(\d+|xx)_)", label);
  contexts["f2"] = string_feature_by_regex(R"(_(\d+|xx)\#)", label);
  contexts["f3"] = string_feature_by_regex(R"(\#(\d+|xx)_)", label);
  contexts["f5"] = string_feature_by_regex(R"(\@(\d+|xx)_)", label);
  contexts["h1"] = string_feature_by_regex(R"(/H\:(\d+|xx)_)", label);
  contexts["i3"] = string_feature_by_regex(R"(\@(\d+|xx)\+)", label);
  contexts["j1"] = string_feature_by_regex(R"(/J\:(\d+|xx)_)", label);
  std::cout << contexts["p3"] << std::endl;

  return new Phoneme(contexts, label);
}

std::string Phoneme::phoneme() { return contexts.at("p3"); }

bool Phoneme::is_pause() { return contexts.at("f1") == "xx"; }

void Mora::set_context(const std::string &key, const std::string &value) const {
  vowel->contexts[key] = value;
  if (consonant != nullptr) consonant->contexts[key] = value;
}

std::vector<Phoneme *> Mora::phonemes() {
  std::vector<Phoneme *> phonemes;
  if (consonant != nullptr) {
    phonemes = {consonant, vowel};
  } else {
    phonemes = {vowel};
  }
  return phonemes;
}

std::vector<std::string> Mora::labels() {
  std::vector<std::string> labels;
  for (Phoneme *phoneme : phonemes()) {
    labels.push_back(phoneme->label);
  }
  return labels;
}

AccentPhrase *AccentPhrase::from_phonemes(std::vector<Phoneme *> phonemes) {
  std::vector<Mora *> moras;
  std::vector<Phoneme *> mora_phonemes;

  for (size_t i = 0; i < phonemes.size(); i++) {
    // workaround for Hihosiba/voicevox_engine#57
    if (phonemes[i]->contexts.at("a2") == "49") break;

    mora_phonemes.push_back(phonemes[i]);
    if (i + 1 == phonemes.size() || phonemes[i]->contexts.at("a2") != phonemes[i + 1]->contexts.at("a2")) {
      Mora *mora;
      if (mora_phonemes.size() == 1) {
        mora = new Mora(mora_phonemes[0]);
      } else if (mora_phonemes.size() == 2) {
        mora = new Mora(mora_phonemes[0], mora_phonemes[1]);
      } else {
        throw std::runtime_error("too long mora");
      }
      moras.push_back(mora);
      mora_phonemes.clear();
    }
  }

  int accent = std::stoi(moras[0]->vowel->contexts.at("f2"));
  bool is_interrogative = moras[moras.size() - 1]->vowel->contexts.at("f3") == "1";
  // workaround for VOICEVOX/voicevox_engine#55
  if (accent > moras.size()) accent = moras.size();
  return new AccentPhrase(moras, accent, is_interrogative);
}

void AccentPhrase::set_context(std::string key, std::string value) {
  for (Mora *mora : moras) mora->set_context(key, value);
}

std::vector<Phoneme *> AccentPhrase::phonemes() {
  std::vector<Phoneme *> phonemes;
  for (Mora *mora : moras) {
    std::vector<Phoneme *> mora_phonemes = mora->phonemes();
    std::copy(mora_phonemes.begin(), mora_phonemes.end(), std::back_inserter(phonemes));
  }
  return phonemes;
}

std::vector<std::string> AccentPhrase::labels() {
  std::vector<std::string> labels;
  for (Phoneme *phoneme : phonemes()) {
    labels.push_back(phoneme->label);
  }
  return labels;
}

AccentPhrase *AccentPhrase::merge(AccentPhrase *accent_phrase) {
  std::vector<Mora *> moras;
  std::copy(this->moras.begin(), this->moras.end(), std::back_inserter(moras));
  std::copy(accent_phrase->moras.begin(), accent_phrase->moras.end(), std::back_inserter(moras));
  return new AccentPhrase(moras, this->accent, accent_phrase->is_interrogative);
}

BreathGroup *BreathGroup::from_phonemes(std::vector<Phoneme *> phonemes) {
  std::vector<AccentPhrase *> accent_phrases;
  std::vector<Phoneme *> accent_phonemes;

  for (size_t i = 0; i < phonemes.size(); i++) {
    accent_phonemes.push_back(phonemes[i]);

    if (i + 1 == phonemes.size() || phonemes[i]->contexts.at("i3") != phonemes[i + 1]->contexts.at("i3") ||
        phonemes[i]->contexts.at("f5") != phonemes[i + 1]->contexts.at("f5")) {
      accent_phrases.push_back(AccentPhrase::from_phonemes(accent_phonemes));
      accent_phonemes.clear();
    }
  }

  return new BreathGroup(accent_phrases);
};

void BreathGroup::set_context(std::string key, std::string value) {
  for (AccentPhrase *accent_phrase : accent_phrases) accent_phrase->set_context(key, value);
}

std::vector<Phoneme *> BreathGroup::phonemes() {
  std::vector<Phoneme *> phonemes;
  for (AccentPhrase *accent_phrase : accent_phrases) {
    std::vector<Phoneme *> accent_phrase_phonemes = accent_phrase->phonemes();
    std::copy(accent_phrase_phonemes.begin(), accent_phrase_phonemes.end(), std::back_inserter(phonemes));
  }
  return phonemes;
}

std::vector<std::string> BreathGroup::labels() {
  std::vector<std::string> labels;
  for (Phoneme *phoneme : phonemes()) {
    labels.push_back(phoneme->label);
  }
  return labels;
}

Utterance Utterance::from_phonemes(const std::vector<Phoneme *> &phonemes) {
  std::vector<BreathGroup *> breath_groups;
  std::vector<Phoneme *> group_phonemes;
  std::vector<Phoneme *> pauses;

  for (Phoneme *phoneme : phonemes) {
    if (!phoneme->is_pause()) {
      group_phonemes.push_back(phoneme);
    } else {
      pauses.push_back(phoneme);

      if (!group_phonemes.empty()) {
        breath_groups.push_back(BreathGroup::from_phonemes(group_phonemes));
        group_phonemes.clear();
      }
    }
  }
  return {breath_groups, pauses};
}

void Utterance::set_context(const std::string &key, const std::string &value) {
  for (BreathGroup *breath_group : breath_groups) breath_group->set_context(key, value);
}

std::vector<Phoneme *> Utterance::phonemes() {
  std::vector<AccentPhrase *> accent_phrases;
  for (BreathGroup *breath_group : breath_groups) {
    std::vector<AccentPhrase *> b_accent_phrases = breath_group->accent_phrases;
    std::copy(b_accent_phrases.begin(), b_accent_phrases.end(), std::back_inserter(accent_phrases));
  }

  std::vector<Phoneme *> phonemes;
  for (size_t i = 0; i < pauses.size(); i++) {
    // if (pauses[i])
    phonemes.push_back(pauses[i]);
    if (i < pauses.size() - 1) {
      std::copy(breath_groups[i]->phonemes().begin(), breath_groups[i]->phonemes().end(), std::back_inserter(phonemes));
    }
  }
  return phonemes;
}

std::vector<std::string> Utterance::labels() {
  std::vector<std::string> labels;
  for (Phoneme *phoneme : phonemes()) {
    labels.push_back(phoneme->label);
  }
  return labels;
}

Utterance extract_full_context_label(OpenJTalk *openjtalk, std::string text) {
  std::vector<std::string> labels = openjtalk->extract_fullcontext(text);
  std::vector<Phoneme *> phonemes;
  for (std::string label : labels) phonemes.push_back(Phoneme::from_label(label));
  return Utterance::from_phonemes(phonemes);
}
}  // namespace voicevox::core::engine
