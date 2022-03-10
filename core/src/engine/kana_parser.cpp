#include "kana_parser.h"

#include <algorithm>
#include <stdexcept>

static const std::map<std::string, MoraModel> text2mora_with_unvoice() {
  std::map<std::string, MoraModel> text2mora_with_unvoice;
  const std::string* mora_list = mora_list_minimum.data();
  int count = 0;
  while (count < mora_list_minimum.size()) {
    std::string text = *mora_list;
    std::optional<std::string> consonant = *(mora_list + 1);
    std::optional<float> consonant_length;
    if (consonant->empty()) {
      consonant = std::nullopt;
      consonant_length = std::nullopt;
    } else {
      consonant_length = 0.0f;
    }

    std::string vowel = *(mora_list + 2);

    MoraModel mora = {text, consonant, consonant_length, vowel, 0.0f, 0.0f};

    text2mora_with_unvoice[text] = mora;

    if (vowel == "a" || vowel == "i" || vowel == "u" || vowel == "e" || vowel == "o") {
      std::string upper_vowel = vowel;
      std::transform(upper_vowel.begin(), upper_vowel.end(), upper_vowel.begin(), ::toupper);
      MoraModel unvoice_mora = {text, consonant, consonant_length, upper_vowel, 0.0f, 0.0f};

      text2mora_with_unvoice[UNVOICE_SYMBOL + text] = unvoice_mora;
    }

    mora_list += 3;
    count += 3;
  }

  return text2mora_with_unvoice;
}

AccentPhraseModel text_to_accent_phrase(std::string phrase) {
  std::optional<unsigned int> accent_index = std::nullopt;

  std::vector<MoraModel> moras;
  int count = 0;

  int base_index = 0;
  std::string stack = "";
  std::optional<std::string> matched_text = std::nullopt;

  const std::map<std::string, MoraModel> text2mora = text2mora_with_unvoice();

  int outer_loop = 0;
  while (base_index < phrase.size()) {
    outer_loop++;
    if (std::string(&phrase[base_index]) == ACCENT_SYMBOL) {
      if (moras.empty()) {
        throw std::runtime_error("accent cannot be set at beginning of accent phrase: " + phrase);
      }
      if (accent_index != std::nullopt) {
        throw std::runtime_error("second accent cannot be set at an accent phrase: " + phrase);
      }

      accent_index = moras.size();
      base_index++;
      continue;
    }
    for (int watch_index = base_index; watch_index < phrase.size(); watch_index++) {
      if (std::string(&phrase[base_index]) == ACCENT_SYMBOL) break;
      stack += phrase[watch_index];
      if (text2mora.find(stack) != text2mora.end()) {
        matched_text = stack;
      }
    }
    if (matched_text == std::nullopt) {
      throw std::runtime_error("unknown text in accent phrase: " + stack);
    } else {
      moras[count] = text2mora.at(*matched_text);
      count++;
      base_index += matched_text->size();
      stack = "";
      matched_text = std::nullopt;
    }
    if (outer_loop > LOOP_LIMIT) throw std::runtime_error("detect infinity loop!");
  }
  if (accent_index == std::nullopt) throw std::runtime_error("accent not found in accent phrase: " + phrase);

  AccentPhraseModel accent_phrase = {
      moras,
      static_cast<unsigned int>(*accent_index),
  };
  return accent_phrase;
}

std::vector<AccentPhraseModel> parse_kana(std::string text) {
  std::vector<AccentPhraseModel> parsed_results;

  std::string phrase = "";
  int count = 0;
  for (size_t i = 0; i <= text.size(); i++) {
    std::string letter = i == text.size() ? "" : &text[i];
    phrase += letter;
    if (i == text.size() || letter == PAUSE_DELIMITER || letter == NOPAUSE_DELIMITER) {
      if (phrase.empty()) {
        throw std::runtime_error("accent phrase at position of " + std::to_string(parsed_results.size() + 1) +
                                 " is empty");
      }
      bool is_interrogative = phrase.find(WIDE_INTERROGATION_MARK) != std::string::npos;
      if (is_interrogative) {
        if (phrase.find(WIDE_INTERROGATION_MARK) != phrase.length() - 1) {
          throw std::runtime_error("interrogative mark cannot be set at not end of accent phrase: " + phrase);
        }
        phrase = phrase.replace(phrase.length() - 1, 1, "");
      }
      AccentPhraseModel accent_phrase = text_to_accent_phrase(phrase);
      if (i < text.size() && letter == PAUSE_DELIMITER) {
        MoraModel pause_mora = {PAUSE_DELIMITER, std::nullopt, std::nullopt, "pau", 0.0f, 0.0f};

        accent_phrase.pause_mora = pause_mora;
      }
      accent_phrase.is_interrogative = is_interrogative;
      parsed_results[count] = accent_phrase;
      count++;
      phrase = "";
    }
  }
  return parsed_results;
}

std::string create_kana(std::vector<AccentPhraseModel> accent_phrases) {
  std::string text = "";
  for (int i = 0; i < accent_phrases.size(); i++) {
    AccentPhraseModel phrase = accent_phrases[i];
    std::vector<MoraModel> moras = phrase.moras;
    for (int j = 0; j < moras.size(); j++) {
      MoraModel mora = moras[j];
      std::string vowel = mora.vowel;
      if (vowel == "A" || vowel == "I" || vowel == "U" || vowel == "E" || vowel == "O") {
        text += UNVOICE_SYMBOL;
      }
      text += mora.text;

      if (j + 1 == phrase.accent) {
        text += ACCENT_SYMBOL;
      }
    }

    if (phrase.is_interrogative) {
      text += WIDE_INTERROGATION_MARK;
    }

    if (i < accent_phrases.size()) {
      if (phrase.pause_mora != std::nullopt)
        text += PAUSE_DELIMITER;
      else
        text += NOPAUSE_DELIMITER;
    }
  }
  return text;
}
