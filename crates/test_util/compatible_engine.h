#include <stdint.h>

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

bool generate_full_intermediate(int64_t length, int64_t phoneme_size,
                                int64_t margin_width, int64_t feature_dim,
                                float *f0, float *phoneme, int64_t *speaker_id,
                                float *output);

bool render_audio_segment(int64_t length, int64_t feature_dim, float *audio_feature,
                          int64_t *speaker_id, float *output);

const char *last_error_message();
