#include <stdint.h>

#include "../voicevox_core_c_api/include/voicevox_core.h"

bool initialize(bool use_gpu, int cpu_num_threads, bool load_all_models);

bool load_model(int64_t speaker_id);

bool is_model_loaded(int64_t speaker_id);

void finalize();

const char *metas();

const char *supported_devices();

bool yukarin_s_forward(int64_t length, int64_t *phoneme_list,
                       int64_t *speaker_id, float *output);

bool yukarin_sa_forward(int64_t length, int64_t *vowel_phoneme_list,
                        int64_t *consonant_phoneme_list,
                        int64_t *start_accent_list, int64_t *end_accent_list,
                        int64_t *start_accent_phrase_list,
                        int64_t *end_accent_phrase_list, int64_t *speaker_id,
                        float *output);

bool decode_forward(int64_t length, int64_t phoneme_size, float *f0,
                    float *phoneme, int64_t *speaker_id, float *output);

const char *last_error_message();

VoicevoxResultCode voicevox_load_openjtalk_dict(const char *dict_path);

VoicevoxResultCode voicevox_tts(const char *text, int64_t speaker_id,
                                int *output_binary_size, uint8_t **output_wav);

VoicevoxResultCode voicevox_tts_from_kana(const char *text, int64_t speaker_id,
                                          int *output_binary_size,
                                          uint8_t **output_wav);

void voicevox_wav_free(uint8_t *wav);

const char *voicevox_error_result_to_message(VoicevoxResultCode result_code);
