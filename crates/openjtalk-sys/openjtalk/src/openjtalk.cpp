#include "openjtalk.hpp"
#include <cstdio>

#include <jpcommon.h>
#include <mecab.h>
#include <njd.h>

#include <cstring>
#include <stdexcept>
#include <string>
#include <vector>

#include <mecab2njd.h>
#include <njd2jpcommon.h>
#include <njd_set_accent_phrase.h>
#include <njd_set_accent_type.h>
#include <njd_set_digit.h>
#include <njd_set_long_vowel.h>
#include <njd_set_pronunciation.h>
#include <njd_set_unvoiced_vowel.h>
#include <text2mecab.h>

namespace {
class OpenJTalk {
public:
  Mecab mecab;
  NJD njd;
  JPCommon jpcommon;

  OpenJTalk() {
    Mecab_initialize(&mecab);
    NJD_initialize(&njd);
    JPCommon_initialize(&jpcommon);
  }

  OpenJTalk(const std::string &dn_mecab) : OpenJTalk() { load(dn_mecab); }

  ~OpenJTalk() { clear(); }

  char **extract_fullcontext(std::string text, int *size);

  void load(const std::string &dn_mecab);
  void clear();
};
} // namespace

namespace {
char **OpenJTalk::extract_fullcontext(std::string text, int *size) {
  char buff[8192];
  text2mecab(buff, text.size(), text.c_str());
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

  int label_size = JPCommon_get_label_size(&jpcommon);
  char **label_feature = JPCommon_get_label_feature(&jpcommon);

  *size = label_size;

  char **labels = (char **)malloc(label_size * sizeof(char *));
  for (int i = 0; i < label_size; i++) {
    labels[i] = strdup(label_feature[i]);
  }

  JPCommon_refresh(&jpcommon);
  NJD_refresh(&njd);
  Mecab_refresh(&mecab);

  return labels;
}

void OpenJTalk::load(const std::string &dn_mecab) {
  BOOL result = Mecab_load(&mecab, dn_mecab.c_str());
  if (result != 1) {
    clear();
    throw std::runtime_error("failed to initialize mecab");
  }
}

void OpenJTalk::clear() {
  Mecab_clear(&mecab);
  NJD_clear(&njd);
  JPCommon_clear(&jpcommon);
}
} // namespace

static OpenJTalk *ojt = NULL;

extern "C" void *OpenJTalk_create() {
  if (ojt == NULL)
    ojt = new OpenJTalk();
  return (void *)ojt;
}

extern "C" char **
OpenJTalk_extract_fullcontext(void *openjtalk, const char *text, size_t *size) {
  int labels_size;
  char **labels =
      ((OpenJTalk *)openjtalk)->extract_fullcontext(text, &labels_size);
  *size = labels_size;
  return labels;
}

extern "C" int OpenJTalk_load(void *openjtalk, const char *dn_mecab) {
  try {
    ((OpenJTalk *)openjtalk)->load(dn_mecab);
  } catch (const std::runtime_error &e) {
    return 1;
  }
  return 0;
}

extern "C" void OpenJTalk_clear(void *openjtalk) {
  ((OpenJTalk *)openjtalk)->clear();
}

extern "C" void OpenJTalk_delete(void *openjtalk) {
  if (ojt != NULL)
    delete ((OpenJTalk *)openjtalk);
  ojt = NULL;
}
