cimport numpy
import numpy

from libcpp cimport bool
from libc.stdint cimport int64_t

cpdef initialize(
    str root_dir_path,
    bool use_gpu,
    int cpu_num_threads = 0
):
    cdef bool success = c_initialize(
        root_dir_path.encode(),
        use_gpu,
        cpu_num_threads
    )
    if not success: raise Exception(c_last_error_message().decode())

cpdef finalize():
    c_finalize()

cpdef metas():
    return c_metas().decode()

cpdef supported_devices():
    return c_supported_devices().decode()

cpdef numpy.ndarray[numpy.float32_t, ndim=1] yukarin_s_forward(
    int64_t length,
    numpy.ndarray[numpy.int64_t, ndim=1] phoneme_list,
    numpy.ndarray[numpy.int64_t, ndim=1] speaker_id,
):
    cdef numpy.ndarray[numpy.float32_t, ndim=1] output = numpy.zeros((length,), dtype=numpy.float32)
    cdef bool success = c_yukarin_s_forward(
        length,
        <int64_t*> phoneme_list.data,
        <int64_t*> speaker_id.data,
        <float*> output.data,
    )
    if not success: raise Exception(c_last_error_message().decode())
    return output


cpdef numpy.ndarray[numpy.float32_t, ndim=2] yukarin_sa_forward(
    int64_t length,
    numpy.ndarray[numpy.int64_t, ndim=2] vowel_phoneme_list,
    numpy.ndarray[numpy.int64_t, ndim=2] consonant_phoneme_list,
    numpy.ndarray[numpy.int64_t, ndim=2] start_accent_list,
    numpy.ndarray[numpy.int64_t, ndim=2] end_accent_list,
    numpy.ndarray[numpy.int64_t, ndim=2] start_accent_phrase_list,
    numpy.ndarray[numpy.int64_t, ndim=2] end_accent_phrase_list,
    numpy.ndarray[numpy.int64_t, ndim=1] speaker_id,
):
    cdef numpy.ndarray[numpy.float32_t, ndim=2] output = numpy.empty((len(speaker_id), length,), dtype=numpy.float32)
    cdef bool success = c_yukarin_sa_forward(
        length,
        <int64_t*> vowel_phoneme_list.data,
        <int64_t*> consonant_phoneme_list.data,
        <int64_t*> start_accent_list.data,
        <int64_t*> end_accent_list.data,
        <int64_t*> start_accent_phrase_list.data,
        <int64_t*> end_accent_phrase_list.data,
        <int64_t*> speaker_id.data,
        <float*> output.data,
    )
    if not success: raise Exception(c_last_error_message().decode())
    return output

cpdef numpy.ndarray[numpy.float32_t, ndim=1] decode_forward(
    int64_t length,
    int64_t phoneme_size,
    numpy.ndarray[numpy.float32_t, ndim=2] f0,
    numpy.ndarray[numpy.float32_t, ndim=2] phoneme,
    numpy.ndarray[numpy.int64_t, ndim=1] speaker_id,
):
    cdef numpy.ndarray[numpy.float32_t, ndim=1] output = numpy.empty((length*256,), dtype=numpy.float32)
    cdef bool success = c_decode_forward(
        length,
        phoneme_size,
        <float*> f0.data,
        <float*> phoneme.data,
        <int64_t*> speaker_id.data,
        <float*> output.data,
    )
    if not success: raise Exception(c_last_error_message().decode())
    return output
