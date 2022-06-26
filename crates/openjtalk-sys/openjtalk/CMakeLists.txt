cmake_minimum_required(VERSION 3.16)
project(OpenJtalkSys)

include(FetchContent)
FetchContent_Declare(
  openjtalk
  GIT_REPOSITORY https://github.com/VOICEVOX/open_jtalk.git
  GIT_TAG 1.11
  SOURCE_SUBDIR src)
FetchContent_MakeAvailable(openjtalk)

install(TARGETS openjtalk LIBRARY DESTINATION lib)
install(
  FILES
    ${openjtalk_SOURCE_DIR}/src/jpcommon/jpcommon.h
    ${openjtalk_SOURCE_DIR}/src/mecab/src/mecab.h
    ${openjtalk_SOURCE_DIR}/src/njd/njd.h
    ${openjtalk_SOURCE_DIR}/src/njd2jpcommon/njd2jpcommon.h
    ${openjtalk_SOURCE_DIR}/src/njd_set_accent_phrase/njd_set_accent_phrase.h
    ${openjtalk_SOURCE_DIR}/src/njd_set_accent_type/njd_set_accent_type.h
    ${openjtalk_SOURCE_DIR}/src/njd_set_digit/njd_set_digit.h
    ${openjtalk_SOURCE_DIR}/src/njd_set_long_vowel/njd_set_long_vowel.h
    ${openjtalk_SOURCE_DIR}/src/njd_set_pronunciation/njd_set_pronunciation.h
    ${openjtalk_SOURCE_DIR}/src/njd_set_unvoiced_vowel/njd_set_unvoiced_vowel.h
    ${openjtalk_SOURCE_DIR}/src/text2mecab/text2mecab.h
    ${openjtalk_SOURCE_DIR}/src/mecab2njd/mecab2njd.h
  DESTINATION include)
