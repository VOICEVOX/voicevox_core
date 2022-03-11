#pragma once

// clang-format off
// For JPCommon's "FILE" type
#include <cstdio>
// clang-format on

#include <jpcommon.h>
#include <mecab.h>
#include <njd.h>

#include <string>
#include <vector>

namespace voicevox::core::engine {
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

  OpenJTalk(const std::string& dn_mecab) : OpenJTalk() { load(dn_mecab); }

  ~OpenJTalk() { clear(); }

  std::vector<std::string> extract_fullcontext(std::string text);

  void load(const std::string& dn_mecab);
  void clear();
};
}  // namespace voicevox::core::engine
