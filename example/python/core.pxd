from libcpp cimport bool

cdef extern from "core.h":
    bool c_initialize "initialize" (
        char *root_dir_path,
        char* yukarin_s_forwarder_path,
        char* yukarin_sa_forwarder_path,
        char* decode_forwarder_path,
        bool use_gpu
    )

    bool c_yukarin_s_forward "yukarin_s_forward" (
        int length,
        long *phoneme_list,
        long *speaker_id,
        float *output
    )

    bool c_yukarin_sa_forward "yukarin_sa_forward" (
        int length,
        long *vowel_phoneme_list,
        long *consonant_phoneme_list,
        long *start_accent_list,
        long *end_accent_list,
        long *start_accent_phrase_list,
        long *end_accent_phrase_list,
        long *speaker_id,
        float *output
    )

    bool c_decode_forward "decode_forward" (
        int length,
        int phoneme_size,
        float *f0,
        float *phoneme,
        long *speaker_id,
        float *output
    )
