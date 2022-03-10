#pragma once

// off clang-format
// For JPCommon's "FILE" type
#include <cstdio>
// on clang-format

#include <jpcommon.h>
#include <mecab.h>
#include <njd.h>

#include <stdexcept>
#include <string>
#include <vector>

class OpenJTalk {
 public:
  Mecab* mecab;
  NJD* njd;
  JPCommon* jpcommon;

  OpenJTalk() {
    mecab = new Mecab();
    njd = new NJD();
    jpcommon = new JPCommon();

    Mecab_initialize(mecab);
    NJD_initialize(njd);
    JPCommon_initialize(jpcommon);
  }

  OpenJTalk(const std::string& dn_mecab) : OpenJTalk() { load(dn_mecab); }

  ~OpenJTalk() { clear(); }

  std::vector<std::string> extract_fullcontext(std::string text) const;

  void load(const std::string& dn_mecab);
  void clear() const;
};
