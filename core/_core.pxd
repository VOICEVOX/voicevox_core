from libcpp cimport bool
from libc.stdint cimport int64_t

cdef extern from "core.h":
    bool c_initialize "initialize" (
        const char *root_dir_path,
        bool use_gpu,
        int cpu_num_threads
    )

    void c_finalize "finalize" ()

    const char *c_metas "metas" ()

    const char *c_supported_devices "supported_devices" ()

    bool c_yukarin_s_forward "yukarin_s_forward" (
        int64_t length,
        int64_t *phoneme_list,
        int64_t *speaker_id,
        float *output
    )

    bool c_yukarin_sa_forward "yukarin_sa_forward" (
        int64_t length,
        int64_t *vowel_phoneme_list,
        int64_t *consonant_phoneme_list,
        int64_t *start_accent_list,
        int64_t *end_accent_list,
        int64_t *start_accent_phrase_list,
        int64_t *end_accent_phrase_list,
        int64_t *speaker_id,
        float *output
    )

    bool c_decode_forward "decode_forward" (
        int64_t length,
        int64_t phoneme_size,
        float *f0,
        float *phoneme,
        int64_t *speaker_id,
        float *output
    )

    const char *c_last_error_message "last_error_message" ()
