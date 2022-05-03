#include "engine/kana_parser.h"

#include <catch2/catch_test_macros.hpp>
#include <catch2/generators/catch_generators.hpp>

using namespace voicevox::core::engine;

TEST_CASE("extract_one_character") {
  struct Given {
    std::string text;
    size_t pos;
  };
  struct Expected {
    std::string one_char;
    size_t after_size;
  };
  struct TestCase {
    std::string name;
    Given given;
    Expected expected;
  };
  auto t = GENERATE(TestCase{.name = "target_is_alphabet",
                             .given = {.text = "abcd", .pos = 2},
                             .expected = {.one_char = "c", .after_size = 1}},
                    TestCase{.name = "target_is_hiragana",
                             .given = {.text = "acあd", .pos = 2},
                             .expected = {.one_char = "あ", .after_size = 3}},
                    TestCase{.name = "target_is_4byte_kanji",
                             .given = {.text = "ace𠀋", .pos = 3},
                             .expected = {.one_char = "𠀋", .after_size = 4}});

  SECTION(t.name) {
    size_t size;
    auto actual_one_char = extract_one_character(t.given.text, t.given.pos, size);
    CHECK(t.expected.one_char == actual_one_char);
    CHECK(t.expected.after_size == size);
  }
}
