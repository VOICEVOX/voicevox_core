#ifndef VOICEVOX_CORE_INCLUDE_GUARD
#define VOICEVOX_CORE_INCLUDE_GUARD

#include <stdint.h>

typedef enum VoicevoxResultCode {
  VOICEVOX_RESULT_SUCCEED = 0,
  VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT = 1,
} VoicevoxResultCode;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

#ifdef _WIN32
__declspec(dllimport)
#endif

bool initialize(bool use_gpu,
                uintptr_t cpu_num_threads,
                bool load_all_models);

#ifdef _WIN32
__declspec(dllimport)
#endif
 bool load_model(int64_t speaker_id);

#ifdef _WIN32
__declspec(dllimport)
#endif
 bool is_model_loaded(int64_t speaker_id);

#ifdef _WIN32
__declspec(dllimport)
#endif
 void finalize(void);

#ifdef _WIN32
__declspec(dllimport)
#endif
 const char *metas(void);

#ifdef _WIN32
__declspec(dllimport)
#endif
 const char *supported_devices(void);

#ifdef _WIN32
__declspec(dllimport)
#endif

enum VoicevoxResultCode yukarin_s_forward(int64_t length,
                                          const int64_t *phoneme_list,
                                          const int64_t *speaker_id,
                                          float *output);

#ifdef _WIN32
__declspec(dllimport)
#endif

enum VoicevoxResultCode yukarin_sa_forward(int64_t length,
                                           const int64_t *vowel_phoneme_list,
                                           const int64_t *consonant_phoneme_list,
                                           const int64_t *start_accent_list,
                                           const int64_t *end_accent_list,
                                           const int64_t *start_accent_phrase_list,
                                           const int64_t *end_accent_phrase_list,
                                           const int64_t *speaker_id,
                                           float *output);

#ifdef _WIN32
__declspec(dllimport)
#endif

enum VoicevoxResultCode decode_forward(int64_t length,
                                       int64_t phoneme_size,
                                       const float *f0,
                                       const float *phoneme,
                                       const int64_t *speaker_id,
                                       float *output);

#ifdef _WIN32
__declspec(dllimport)
#endif

enum VoicevoxResultCode voicevox_tts(const char *text,
                                     int64_t speaker_id,
                                     uintptr_t *output_binary_size,
                                     uint8_t *const *output_wav);

#ifdef _WIN32
__declspec(dllimport)
#endif

enum VoicevoxResultCode voicevox_tts_from_kana(const char *text,
                                               int64_t speaker_id,
                                               uintptr_t *output_binary_size,
                                               uint8_t *const *output_wav);

#ifdef _WIN32
__declspec(dllimport)
#endif

const char *voicevox_error_result_to_message(enum VoicevoxResultCode result_code);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* VOICEVOX_CORE_INCLUDE_GUARD */
