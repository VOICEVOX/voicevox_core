#include "openjtalk.h"

#include <mecab2njd.h>
#include <njd2jpcommon.h>
#include <njd_set_accent_phrase.h>
#include <njd_set_accent_type.h>
#include <njd_set_digit.h>
#include <njd_set_long_vowel.h>
#include <njd_set_pronunciation.h>
#include <njd_set_unvoiced_vowel.h>
#include <text2mecab.h>

#include <stdexcept>

namespace voicevox::core::engine {
std::vector<std::string> OpenJTalk::extract_fullcontext(std::string text) {
  char buff[8192];
  text2mecab(buff, text.c_str());
  Mecab_analysis(&mecab, buff);
  mecab2njd(&njd, Mecab_get_feature(&mecab), Mecab_get_size(&mecab));
  njd_set_pronunciation(&njd);
  njd_set_digit(&njd);
  njd_set_accent_phrase(&njd);
  njd_set_accent_type(&njd);
  njd_set_unvoiced_vowel(&njd);
  njd_set_long_vowel(&njd);
  njd2jpcommon(&jpcommon, &njd);
  JPCommon_make_label(&jpcommon);

  std::vector<std::string> labels;

  int label_size = JPCommon_get_label_size(&jpcommon);
  char** label_feature = JPCommon_get_label_feature(&jpcommon);

  labels.clear();
  for (int i = 0; i < label_size; i++) labels.push_back(label_feature[i]);

  JPCommon_refresh(&jpcommon);
  NJD_refresh(&njd);
  Mecab_refresh(&mecab);

  return labels;
}

void OpenJTalk::load(const std::string& dn_mecab) {
  BOOL result = Mecab_load(&mecab, dn_mecab.c_str());
  if (result != 1) {
    clear();
    throw std::runtime_error("failed to initialize mecab");
  }
  dict_loaded = true;
}

void OpenJTalk::clear() {
  Mecab_clear(&mecab);
  NJD_clear(&njd);
  JPCommon_clear(&jpcommon);
}
}  // namespace voicevox::core::engine
