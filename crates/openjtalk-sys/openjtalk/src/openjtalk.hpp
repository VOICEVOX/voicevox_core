#pragma once

#include <cstddef>

extern "C" void *OpenJTalk_create();
extern "C" char **OpenJTalk_extract_fullcontext(void *openjtalk,
                                                const char *text, size_t *size);
extern "C" int OpenJTalk_load(void *openjtalk, const char *dn_mecab);
extern "C" void OpenJTalk_clear(void *openjtalk);
extern "C" void OpenJTalk_delete(void *openjtalk);
