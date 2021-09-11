cimport numpy
import numpy

from libcpp cimport bool

cpdef initialize(
    str root_dir_path,
    bool use_gpu,
):
    cdef bool success = c_initialize(
        root_dir_path.encode(),
        use_gpu,
    )
    assert success

cpdef metas():
    return c_metas().decode()

cpdef numpy.ndarray[numpy.float32_t, ndim=1] yukarin_s_forward(
    int length,
    numpy.ndarray[numpy.int64_t, ndim=1] phoneme_list,
    numpy.ndarray[numpy.int64_t, ndim=1] speaker_id,
):
    cdef numpy.ndarray[numpy.float32_t, ndim=1] output = numpy.zeros((length,), dtype=numpy.float32)
    cdef bool success = c_yukarin_s_forward(
        length,
        <long*> phoneme_list.data,
        <long*> speaker_id.data,
        <float*> output.data,
    )
    assert success
    return output


cpdef numpy.ndarray[numpy.float32_t, ndim=2] yukarin_sa_forward(
    int length,
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
        <long*> vowel_phoneme_list.data,
        <long*> consonant_phoneme_list.data,
        <long*> start_accent_list.data,
        <long*> end_accent_list.data,
        <long*> start_accent_phrase_list.data,
        <long*> end_accent_phrase_list.data,
        <long*> speaker_id.data,
        <float*> output.data,
    )
    assert success
    return output

cpdef numpy.ndarray[numpy.float32_t, ndim=1] decode_forward(
    int length,
    int phoneme_size,
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
        <long*> speaker_id.data,
        <float*> output.data,
    )
    assert success
    return output
