#include "kana_parser.h"

#include <algorithm>
#include <optional>
#include <stdexcept>

#include "mora_list.h"

namespace voicevox::core::engine {
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

template <typename N>
std::string extract_one_character(const std::string& text, N pos, N* size) {
  // UTF-8の文字は可変長なので、leadの値で長さを判別する
  unsigned char lead = text[pos];

  if (lead < 0x80) {
    *size = 1;
  } else if (lead < 0xE0) {
    *size = 2;
  } else if (lead < 0xF0) {
    *size = 3;
  } else {
    *size = 4;
  }

  return text.substr(pos, *size);
}

AccentPhraseModel text_to_accent_phrase(std::string phrase) {
  std::optional<unsigned int> accent_index = std::nullopt;

  std::vector<MoraModel> moras;

  int base_index = 0;
  std::string stack = "";
  std::optional<std::string> matched_text = std::nullopt;

  const std::map<std::string, MoraModel> text2mora = text2mora_with_unvoice();

  int outer_loop = 0;
  while (base_index < phrase.size()) {
    outer_loop++;
    int char_size;
    std::string letter = extract_one_character(phrase, base_index, &char_size);
    if (letter == ACCENT_SYMBOL) {
      if (moras.empty()) {
        throw std::runtime_error("accent cannot be set at beginning of accent phrase: " + phrase);
      }
      if (accent_index != std::nullopt) {
        throw std::runtime_error("second accent cannot be set at an accent phrase: " + phrase);
      }

      accent_index = moras.size();
      base_index += char_size;
      continue;
    }
    int watch_char_size;
    for (int watch_index = base_index; watch_index < phrase.size(); watch_index += watch_char_size) {
      std::string watch_letter = extract_one_character(phrase, watch_index, &watch_char_size);
      if (watch_letter == ACCENT_SYMBOL) break;
      stack += watch_letter;
      if (text2mora.find(stack) != text2mora.end()) {
        matched_text = stack;
      }
    }
    if (matched_text == std::nullopt) {
      throw std::runtime_error("unknown text in accent phrase: " + stack);
    } else {
      moras.push_back(text2mora.at(*matched_text));
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

  size_t char_size;
  for (size_t pos = 0; pos <= text.size(); pos += char_size) {
    std::string letter;
    if (pos != text.size()) {
      letter = extract_one_character(text, pos, &char_size);
    }
    phrase += letter;
    if (pos == text.size() || letter == PAUSE_DELIMITER || letter == NOPAUSE_DELIMITER) {
      if (phrase.empty()) {
        throw std::runtime_error("accent phrase at position of " + std::to_string(parsed_results.size() + 1) +
                                 " is empty");
      }
      bool is_interrogative = phrase.find(WIDE_INTERROGATION_MARK) != std::string::npos;
      if (is_interrogative) {
        if (phrase.find(WIDE_INTERROGATION_MARK) != phrase.length() - char_size) {
          throw std::runtime_error("interrogative mark cannot be set at not end of accent phrase: " + phrase);
        }
        phrase = phrase.replace(phrase.length() - char_size, char_size, "");
      }
      AccentPhraseModel accent_phrase = text_to_accent_phrase(phrase);
      if (pos < text.size() && letter == PAUSE_DELIMITER) {
        MoraModel pause_mora = {PAUSE_DELIMITER, std::nullopt, std::nullopt, "pau", 0.0f, 0.0f};

        accent_phrase.pause_mora = pause_mora;
      }
      accent_phrase.is_interrogative = is_interrogative;
      parsed_results.push_back(accent_phrase);
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
}  // namespace voicevox::core::engine
