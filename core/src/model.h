constexpr char metasJson[] =
#include <metas.json>
    ;

const uint8_t yukarin_s_model[] = {
#include <yukarin_s.onnx.txt>
};

const uint8_t yukarin_sa_model[] = {
#include <yukarin_sa.onnx.txt>
};

const uint8_t decode_model[] = {
#include <decode.onnx.txt>
};