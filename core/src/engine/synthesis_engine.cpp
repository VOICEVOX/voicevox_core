#include "synthesis_engine.h"

#include <algorithm>
#include <cmath>
#include <iterator>
#include <optional>
#include <sstream>
#include <stdexcept>

#include "../core.h"
#include "full_context_label.h"
#include "mora_list.h"

namespace voicevox::core::engine {
std::vector<MoraModel> to_flatten_moras(std::vector<AccentPhraseModel> accent_phrases) {
  std::vector<MoraModel> flatten_moras;

  for (AccentPhraseModel accent_phrase : accent_phrases) {
    std::vector<MoraModel> moras = accent_phrase.moras;
    for (MoraModel mora : moras) {
      flatten_moras.push_back(mora);
    }
    if (accent_phrase.pause_mora != std::nullopt) {
      MoraModel pause_mora = static_cast<MoraModel>(*accent_phrase.pause_mora);
      flatten_moras.push_back(pause_mora);
    }
  }

  return flatten_moras;
}

std::vector<OjtPhoneme> to_phoneme_data_list(std::vector<std::string> phoneme_str_list) {
  std::vector<OjtPhoneme> phoneme_data_list;
  for (size_t i = 0; i < phoneme_str_list.size(); i++) {
    phoneme_data_list.push_back(OjtPhoneme(phoneme_str_list[i], (float)i, (float)i + 1.0f));
  }
  return OjtPhoneme::convert(phoneme_data_list);
}

void split_mora(std::vector<OjtPhoneme> phoneme_list, std::vector<OjtPhoneme> &consonant_phoneme_list,
                std::vector<OjtPhoneme> &vowel_phoneme_list, std::vector<int64_t> &vowel_indexes) {
  for (size_t i = 0; i < phoneme_list.size(); i++) {
    std::vector<std::string>::iterator result =
        std::find(mora_phoneme_list.begin(), mora_phoneme_list.end(), phoneme_list[i].phoneme);
    if (result != mora_phoneme_list.end()) {
      vowel_indexes.push_back((long)i);
    }
  }
  for (int64_t index : vowel_indexes) {
    vowel_phoneme_list.push_back(phoneme_list[index]);
  }
  consonant_phoneme_list.push_back(OjtPhoneme());
  for (size_t i = 0; i < vowel_indexes.size() - 1; i++) {
    int64_t prev = vowel_indexes[i];
    int64_t next = vowel_indexes[1 + i];
    if (next - prev == 1) {
      consonant_phoneme_list.push_back(OjtPhoneme());
    } else {
      consonant_phoneme_list.push_back(phoneme_list[next - 1]);
    }
  }
}

std::vector<AccentPhraseModel> adjust_interrogative_accent_phrases(std::vector<AccentPhraseModel> accent_phrases) {
  std::vector<AccentPhraseModel> new_accent_phrases(accent_phrases.size());
  for (size_t i = 0; i < accent_phrases.size(); i++) {
    AccentPhraseModel accent_phrase = accent_phrases[i];
    AccentPhraseModel new_accent_phrase = {
        adjust_interrogative_moras(accent_phrase),
        accent_phrase.accent,
        accent_phrase.pause_mora,
        accent_phrase.is_interrogative,
    };
    new_accent_phrases[i] = new_accent_phrase;
  }
  return new_accent_phrases;
}

std::vector<MoraModel> adjust_interrogative_moras(AccentPhraseModel accent_phrase) {
  std::vector<MoraModel> moras = accent_phrase.moras;
  if (accent_phrase.is_interrogative) {
    if (!moras.empty()) {
      MoraModel last_mora = moras[moras.size() - 1];
      float last_mora_pitch = last_mora.pitch;
      if (last_mora_pitch != 0.0) {
        std::vector<MoraModel> new_moras(moras.size() + 1);
        std::copy(moras.begin(), moras.end(), new_moras.begin());
        MoraModel interrogative_mora = make_interrogative_mora(last_mora);
        new_moras[moras.size()] = interrogative_mora;
        return new_moras;
      }
    }
  }
  return moras;
}

MoraModel make_interrogative_mora(MoraModel last_mora) {
  float fix_vowel_length = 0.15f;
  float adjust_pitch = 0.3f;
  float max_pitch = 6.5f;

  float pitch = last_mora.pitch + adjust_pitch;
  if (pitch > max_pitch) {
    pitch = max_pitch;
  }
  MoraModel interrogative_mora = {
      mora2text(last_mora.vowel), std::nullopt, std::nullopt, last_mora.vowel, fix_vowel_length, pitch,
  };
  return interrogative_mora;
}

std::vector<AccentPhraseModel> SynthesisEngine::create_accent_phrases(std::string text, int64_t *speaker_id) {
  if (text.empty()) {
    return {};
  }

  Utterance utterance = extract_full_context_label(*m_openjtalk, text);
  if (utterance.breath_groups.empty()) {
    return {};
  }

  size_t accent_phrases_size = 0;
  for (const auto &breath_group : utterance.breath_groups) accent_phrases_size += breath_group.accent_phrases.size();
  std::vector<AccentPhraseModel> accent_phrases(accent_phrases_size);

  int accent_phrases_count = 0;
  for (size_t i = 0; i < utterance.breath_groups.size(); i++) {
    const auto &breath_group = utterance.breath_groups[i];
    for (size_t j = 0; j < breath_group.accent_phrases.size(); j++) {
      const auto &accent_phrase = breath_group.accent_phrases[j];

      std::vector<MoraModel> moras(accent_phrase.moras.size());
      for (size_t k = 0; k < accent_phrase.moras.size(); k++) {
        auto &mora = accent_phrase.moras[k];
        std::string moras_text = "";
        for (auto &phoneme : mora.phonemes()) moras_text += phoneme.phoneme();
        std::transform(moras_text.begin(), moras_text.end(), moras_text.begin(), ::tolower);
        if (moras_text == "n") moras_text = "N";
        std::optional<std::string> consonant = std::nullopt;
        std::optional<float> consonant_length = std::nullopt;
        if (mora.consonant.has_value()) {
          consonant = mora.consonant.value().phoneme();
          consonant_length = 0.0f;
        }
        MoraModel new_mora = {
            mora2text(moras_text), consonant, consonant_length, mora.vowel.phoneme(), 0.0f, 0.0f,
        };
        moras[k] = new_mora;
      }

      std::optional<MoraModel> pause_mora = std::nullopt;
      if (i != utterance.breath_groups.size() - 1 && j == breath_group.accent_phrases.size() - 1) {
        pause_mora = {
            "、", std::nullopt, std::nullopt, "pau", 0.0f, 0.0f,
        };
      }
      AccentPhraseModel new_accent_phrase = {
          moras,
          accent_phrase.accent,
          pause_mora,
          accent_phrase.is_interrogative,
      };

      accent_phrases[accent_phrases_count] = new_accent_phrase;
      accent_phrases_count++;
    }
  }

  accent_phrases = replace_mora_data(accent_phrases, speaker_id);

  return accent_phrases;
}

std::vector<AccentPhraseModel> SynthesisEngine::replace_mora_data(std::vector<AccentPhraseModel> accent_phrases,
                                                                  int64_t *speaker_id) {
  return replace_mora_pitch(replace_phoneme_length(accent_phrases, speaker_id), speaker_id);
}

std::vector<AccentPhraseModel> SynthesisEngine::replace_phoneme_length(std::vector<AccentPhraseModel> accent_phrases,
                                                                       int64_t *speaker_id) {
  std::vector<MoraModel> flatten_moras;
  std::vector<std::string> phoneme_str_list;
  std::vector<OjtPhoneme> phoneme_data_list;
  initial_process(accent_phrases, flatten_moras, phoneme_str_list, phoneme_data_list);

  std::vector<OjtPhoneme> consonant_phoneme_list;
  std::vector<OjtPhoneme> vowel_phoneme_list;
  std::vector<int64_t> vowel_indexes_data;
  split_mora(phoneme_data_list, consonant_phoneme_list, vowel_phoneme_list, vowel_indexes_data);

  std::vector<int64_t> phoneme_list_s;
  for (OjtPhoneme phoneme_data : phoneme_data_list) phoneme_list_s.push_back(phoneme_data.phoneme_id());
  std::vector<float> phoneme_length(phoneme_list_s.size(), 0.0);
  bool success = yukarin_s_forward((int64_t)phoneme_list_s.size(), (int64_t *)phoneme_list_s.data(), speaker_id,
                                   phoneme_length.data());

  if (!success) {
    throw std::runtime_error(last_error_message());
  }

  int index = 0;
  for (size_t i = 0; i < accent_phrases.size(); i++) {
    AccentPhraseModel accent_phrase = accent_phrases[i];
    std::vector<MoraModel> moras = accent_phrase.moras;
    for (size_t j = 0; j < moras.size(); j++) {
      MoraModel mora = moras[j];
      if (mora.consonant != std::nullopt) {
        mora.consonant_length = phoneme_length[vowel_indexes_data[index + 1] - 1];
      }
      mora.vowel_length = phoneme_length[vowel_indexes_data[index + 1]];
      index++;
      moras[j] = mora;
    }
    accent_phrase.moras = moras;
    if (accent_phrase.pause_mora != std::nullopt) {
      std::optional<MoraModel> pause_mora = accent_phrase.pause_mora;
      pause_mora->vowel_length = phoneme_length[vowel_indexes_data[index + 1]];
      index++;
      accent_phrase.pause_mora = pause_mora;
    }
    accent_phrases[i] = accent_phrase;
  }

  return accent_phrases;
}

std::vector<AccentPhraseModel> SynthesisEngine::replace_mora_pitch(std::vector<AccentPhraseModel> accent_phrases,
                                                                   int64_t *speaker_id) {
  std::vector<MoraModel> flatten_moras;
  std::vector<std::string> phoneme_str_list;
  std::vector<OjtPhoneme> phoneme_data_list;
  initial_process(accent_phrases, flatten_moras, phoneme_str_list, phoneme_data_list);

  std::vector<int64_t> base_start_accent_list;
  std::vector<int64_t> base_end_accent_list;
  std::vector<int64_t> base_start_accent_phrase_list;
  std::vector<int64_t> base_end_accent_phrase_list;

  base_start_accent_list.push_back(0);
  base_end_accent_list.push_back(0);
  base_start_accent_phrase_list.push_back(0);
  base_end_accent_phrase_list.push_back(0);
  for (AccentPhraseModel accent_phrase : accent_phrases) {
    unsigned int accent = accent_phrase.accent == 1 ? 0 : 1;
    create_one_accent_list(base_start_accent_list, accent_phrase, (int)accent);

    accent = accent_phrase.accent - 1;
    create_one_accent_list(base_end_accent_list, accent_phrase, (int)accent);

    create_one_accent_list(base_start_accent_phrase_list, accent_phrase, 0);

    create_one_accent_list(base_end_accent_phrase_list, accent_phrase, -1);
  }
  base_start_accent_list.push_back(0);
  base_end_accent_list.push_back(0);
  base_start_accent_phrase_list.push_back(0);
  base_end_accent_phrase_list.push_back(0);

  std::vector<OjtPhoneme> consonant_phoneme_data_list;
  std::vector<OjtPhoneme> vowel_phoneme_data_list;
  std::vector<int64_t> vowel_indexes;
  split_mora(phoneme_data_list, consonant_phoneme_data_list, vowel_phoneme_data_list, vowel_indexes);

  std::vector<int64_t> consonant_phoneme_list;
  for (OjtPhoneme consonant_phoneme_data : consonant_phoneme_data_list) {
    consonant_phoneme_list.push_back(consonant_phoneme_data.phoneme_id());
  }

  std::vector<int64_t> vowel_phoneme_list;
  for (OjtPhoneme vowel_phoneme_data : vowel_phoneme_data_list) {
    vowel_phoneme_list.push_back(vowel_phoneme_data.phoneme_id());
  }

  std::vector<int64_t> start_accent_list;
  std::vector<int64_t> end_accent_list;
  std::vector<int64_t> start_accent_phrase_list;
  std::vector<int64_t> end_accent_phrase_list;

  for (int64_t vowel_index : vowel_indexes) {
    start_accent_list.push_back(base_start_accent_list[vowel_index]);
    end_accent_list.push_back(base_end_accent_list[vowel_index]);
    start_accent_phrase_list.push_back(base_start_accent_phrase_list[vowel_index]);
    end_accent_phrase_list.push_back(base_end_accent_phrase_list[vowel_index]);
  }

  int64_t length = vowel_phoneme_list.size();
  std::vector<float> f0_list(length, 0);
  bool success = yukarin_sa_forward(length, vowel_phoneme_list.data(), consonant_phoneme_list.data(),
                                    start_accent_list.data(), end_accent_list.data(), start_accent_phrase_list.data(),
                                    end_accent_phrase_list.data(), speaker_id, f0_list.data());

  if (!success) {
    throw std::runtime_error(last_error_message());
  }

  for (size_t i = 0; i < vowel_phoneme_data_list.size(); i++) {
    std::vector<std::string>::iterator found_unvoice_mora = std::find(
        unvoiced_mora_phoneme_list.begin(), unvoiced_mora_phoneme_list.end(), vowel_phoneme_data_list[i].phoneme);
    if (found_unvoice_mora != unvoiced_mora_phoneme_list.end()) f0_list[i] = 0;
  }

  int index = 0;
  for (size_t i = 0; i < accent_phrases.size(); i++) {
    AccentPhraseModel accent_phrase = accent_phrases[i];
    std::vector<MoraModel> moras = accent_phrase.moras;
    for (size_t j = 0; j < moras.size(); j++) {
      MoraModel mora = moras[j];
      mora.pitch = f0_list[index + 1];
      index++;
      moras[j] = mora;
    }
    accent_phrase.moras = moras;
    if (accent_phrase.pause_mora != std::nullopt) {
      std::optional<MoraModel> pause_mora = accent_phrase.pause_mora;
      pause_mora->pitch = f0_list[index + 1];
      index++;
      accent_phrase.pause_mora = pause_mora;
    }
    accent_phrases[i] = accent_phrase;
  }

  return accent_phrases;
}

std::vector<uint8_t> SynthesisEngine::synthesis_wave_format(AudioQueryModel query, int64_t *speaker_id,
                                                            int *binary_size, bool enable_interrogative_upspeak) {
  std::vector<float> wave = synthesis(query, speaker_id, enable_interrogative_upspeak);

  float volume_scale = query.volume_scale;
  bool output_stereo = query.output_stereo;
  // TODO: 44.1kHzなどの対応
  unsigned int output_sampling_rate = query.output_sampling_rate;

  char num_channels = output_stereo ? 2 : 1;
  char bit_depth = 16;
  uint32_t repeat_count = (output_sampling_rate / default_sampling_rate) * num_channels;
  char block_size = bit_depth * num_channels / 8;

  std::stringstream ss;
  ss.write("RIFF", 4);
  int bytes_size = (int)wave.size() * repeat_count * 8;
  int wave_size = bytes_size + 44 - 8;
  for (int i = 0; i < 4; i++) {
    ss.put((uint8_t)(wave_size & 0xff));  // chunk size
    wave_size >>= 8;
  }
  ss.write("WAVEfmt ", 8);

  ss.put((char)16);                                // fmt header length
  for (int i = 0; i < 3; i++) ss.put((uint8_t)0);  // fmt header length
  ss.put(1);                                       // linear PCM
  ss.put(0);                                       // linear PCM
  ss.put(num_channels);                            // channnel
  ss.put(0);                                       // channnel

  int sampling_rate = output_sampling_rate;
  for (int i = 0; i < 4; i++) {
    ss.put((char)(sampling_rate & 0xff));
    sampling_rate >>= 8;
  }
  int block_rate = output_sampling_rate * block_size;
  for (int i = 0; i < 4; i++) {
    ss.put((char)(block_rate & 0xff));
    block_rate >>= 8;
  }

  ss.put(block_size);
  ss.put(0);
  ss.put(bit_depth);
  ss.put(0);

  ss.write("data", 4);
  size_t data_p = ss.tellp();
  for (int i = 0; i < 4; i++) {
    ss.put((char)(bytes_size & 0xff));
    block_rate >>= 8;
  }

  for (size_t i = 0; i < wave.size(); i++) {
    float v = wave[i] * volume_scale;
    // clip
    v = 1.0f < v ? 1.0f : v;
    v = -1.0f > v ? -1.0f : v;
    int16_t data = (int16_t)(v * (float)0x7fff);
    for (uint32_t j = 0; j < repeat_count; j++) {
      ss.put((char)(data & 0xff));
      ss.put((char)((data & 0xff00) >> 8));
    }
  }

  size_t last_p = ss.tellp();
  last_p -= 8;
  ss.seekp(4);
  for (int i = 0; i < 4; i++) {
    ss.put((char)(last_p & 0xff));
    last_p >>= 8;
  }
  ss.seekp(data_p);
  size_t pointer = last_p - data_p - 4;
  for (int i = 0; i < 4; i++) {
    ss.put((char)(pointer & 0xff));
    pointer >>= 8;
  }

  ss.seekg(0, std::ios::end);
  *binary_size = (int)ss.tellg();
  ss.seekg(0, std::ios::beg);

  std::vector<uint8_t> result(*binary_size);
  for (int i = 0; i < *binary_size; i++) {
    result[i] = (uint8_t)ss.get();
  }
  return result;
}

std::vector<float> SynthesisEngine::synthesis(AudioQueryModel query, int64_t *speaker_id,
                                              bool enable_interrogative_upspeak) {
  std::vector<AccentPhraseModel> accent_phrases = query.accent_phrases;
  if (enable_interrogative_upspeak) {
    accent_phrases = adjust_interrogative_accent_phrases(accent_phrases);
  }
  std::vector<MoraModel> flatten_moras;
  std::vector<std::string> phoneme_str_list;
  std::vector<OjtPhoneme> phoneme_data_list;
  initial_process(accent_phrases, flatten_moras, phoneme_str_list, phoneme_data_list);

  float pre_phoneme_length = query.pre_phoneme_length;
  float post_phoneme_length = query.post_phoneme_length;

  float pitch_scale = query.pitch_scale;
  float speed_scale = query.speed_scale;
  float intonation_scale = query.intonation_scale;

  std::vector<float> phoneme_length_list;
  phoneme_length_list.push_back(pre_phoneme_length);

  std::vector<float> f0_list;
  std::vector<bool> voiced;
  f0_list.push_back(0.0);
  voiced.push_back(false);
  float mean_f0 = 0.0;
  int count = 0;

  for (MoraModel mora : flatten_moras) {
    if (mora.consonant != std::nullopt) {
      phoneme_length_list.push_back(static_cast<float>(*mora.consonant_length));
    }
    phoneme_length_list.push_back(mora.vowel_length);
    float f0_single = mora.pitch * std::pow(2.0f, pitch_scale);
    f0_list.push_back(f0_single);
    bool big_than_zero = f0_single > 0.0;
    voiced.push_back(big_than_zero);
    if (big_than_zero) {
      mean_f0 += f0_single;
      count++;
    }
  }
  phoneme_length_list.push_back(post_phoneme_length);
  f0_list.push_back(0.0);
  mean_f0 /= (float)count;

  if (!std::isnan(mean_f0)) {
    for (size_t i = 0; i < f0_list.size(); i++) {
      if (voiced[i]) {
        f0_list[i] = (f0_list[i] - mean_f0) * intonation_scale + mean_f0;
      }
    }
  }

  std::vector<OjtPhoneme> consonant_phoneme_data_list;
  std::vector<OjtPhoneme> vowel_phoneme_data_list;
  std::vector<int64_t> vowel_indexes;
  split_mora(phoneme_data_list, consonant_phoneme_data_list, vowel_phoneme_data_list, vowel_indexes);

  std::vector<std::vector<float>> phoneme;
  std::vector<float> f0;
  float rate = 24000.0 / 256.0;
  int phoneme_length_sum = 0;
  int f0_count = 0;
  int64_t *p_vowel_index = vowel_indexes.data();
  for (size_t i = 0; i < phoneme_length_list.size(); i++) {
    int phoneme_length = (int)std::round((std::round(phoneme_length_list[i] * rate) / speed_scale));
    long phoneme_id = phoneme_data_list[i].phoneme_id();
    for (int j = 0; j < phoneme_length; j++) {
      std::vector<float> phonemes_vector(OjtPhoneme::num_phoneme(), 0.0);
      phonemes_vector[phoneme_id] = 1;
      phoneme.push_back(phonemes_vector);
    }
    phoneme_length_sum += phoneme_length;
    if ((int64_t)i == *p_vowel_index) {
      for (long k = 0; k < phoneme_length_sum; k++) {
        f0.push_back(f0_list[f0_count]);
      }
      f0_count++;
      phoneme_length_sum = 0;
      p_vowel_index++;
    }
  }

  // 2次元のvectorを1次元に変換し、アドレスを連続させる
  std::vector<float> flatten_phoneme;
  for (std::vector<float> &p : phoneme) {
    std::copy(p.begin(), p.end(), std::back_inserter(flatten_phoneme));
  }
  std::vector<float> wave(f0.size() * 256, 0.0);
  bool success = decode_forward((int64_t)f0.size(), OjtPhoneme::num_phoneme(), f0.data(), flatten_phoneme.data(),
                                speaker_id, wave.data());

  if (!success) {
    throw std::runtime_error(last_error_message());
  }

  return wave;
}

void SynthesisEngine::load_openjtalk_dict(const std::string &dict_path) { m_openjtalk.load(dict_path); }

void SynthesisEngine::initial_process(std::vector<AccentPhraseModel> &accent_phrases,
                                      std::vector<MoraModel> &flatten_moras, std::vector<std::string> &phoneme_str_list,
                                      std::vector<OjtPhoneme> &phoneme_data_list) {
  flatten_moras = to_flatten_moras(accent_phrases);

  phoneme_str_list.push_back("pau");
  for (MoraModel mora : flatten_moras) {
    std::optional<std::string> consonant = mora.consonant;
    if (consonant != std::nullopt) phoneme_str_list.push_back(static_cast<std::string>(*consonant));
    phoneme_str_list.push_back(mora.vowel);
  }
  phoneme_str_list.push_back("pau");

  phoneme_data_list = to_phoneme_data_list(phoneme_str_list);
}

void SynthesisEngine::create_one_accent_list(std::vector<int64_t> &accent_list, AccentPhraseModel accent_phrase,
                                             int point) {
  std::vector<MoraModel> moras = accent_phrase.moras;

  std::vector<int64_t> one_accent_list;
  for (size_t i = 0; i < moras.size(); i++) {
    MoraModel mora = moras[i];
    int64_t value;
    if ((int)i == point || (point < 0 && i == moras.size() + point))
      value = 1;
    else
      value = 0;
    one_accent_list.push_back(value);
    if (mora.consonant != std::nullopt) {
      one_accent_list.push_back(value);
    }
  }
  if (accent_phrase.pause_mora != std::nullopt) one_accent_list.push_back(0);
  std::copy(one_accent_list.begin(), one_accent_list.end(), std::back_inserter(accent_list));
}
}  // namespace voicevox::core::engine
