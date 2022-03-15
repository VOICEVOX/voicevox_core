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

  ~OpenJTalk() { clear(); }

  std::vector<std::string> extract_fullcontext(std::string text);

  void load(const std::string& dn_mecab);
  void clear();
  bool is_dict_loaded() const { return dict_loaded; }

 private:
  bool dict_loaded = false;

  // OpenJTalk内で管理しているfieldはOpenJTalkのオブジェクトがコピーできてしまうと二重でクリアされたり、メモリリークの危険性がある
  // そのためCopy Protectionによりコピーを防ぐ
  OpenJTalk(const OpenJTalk&) = delete;
  void operator=(const OpenJTalk&) = delete;
};
}  // namespace voicevox::core::engine
