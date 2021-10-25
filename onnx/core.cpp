#include <onnxruntime_cxx_api.h>

#include <array>
#include <filesystem>
#include <memory>
#include <string>

#define VOICEVOX_CORE_EXPORTS
#include "core.h"

#define NOT_INITIALIZED_ERR "Call initialize() first."
#define NOT_FOUND_ERR "No such file or directory: "
#define ONNX_ERR "ONNX raise exception: "

namespace fs = std::filesystem;
constexpr std::array<int64_t, 0> scalar_shape{};
constexpr std::array<int64_t, 1> speaker_shape{1};

struct Status {
  Status(const char *root_dir_path_utf8, bool use_gpu_)
      : root_dir_path(root_dir_path_utf8),
        use_gpu(use_gpu_),
        memory_info(Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeCPU)) {
    // deprecated in C++20; Use char8_t for utf-8 char in the future.
    fs::path root = fs::u8path(root_dir_path);
    Ort::SessionOptions session_options;
#ifdef USE_CUDA
    if (use_gpu) {
      Ort::ThrowOnError(OrtSessionOptionsAppendExecutionProvider_CUDA(session_options, 0));
    }
#endif
    yukarin_s = std::make_shared<Ort::Session>(env, (root / "yukarin_s.onnx").c_str(), session_options);
    yukarin_sa = std::make_shared<Ort::Session>(env, (root / "yukarin_sa.onnx").c_str(), session_options);
    decode = std::make_shared<Ort::Session>(env, (root / "decode.onnx").c_str(), session_options);
  }

  std::string root_dir_path;
  bool use_gpu;
  Ort::MemoryInfo memory_info;

  Ort::Env env{ORT_LOGGING_LEVEL_ERROR};
  std::shared_ptr<Ort::Session> yukarin_s, yukarin_sa, decode;
};

static std::string error_message;
static bool initialized = false;
static std::unique_ptr<Status> status;

template <typename T, size_t Rank>
Ort::Value ToTensor(T *data, const std::array<int64_t, Rank> &shape) {
  int64_t count = 1;
  for (int64_t dim : shape) {
    count *= dim;
  }
  return Ort::Value::CreateTensor<T>(status->memory_info, data, count, shape.data(), shape.size());
}

bool initialize(const char *root_dir_path, bool use_gpu) {
  try {
    status = std::make_unique<Status>(root_dir_path, use_gpu);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  initialized = true;
  return true;
}

const char *metas() { throw std::runtime_error("NotImplemented"); }

bool yukarin_s_forward(int length, long *phoneme_list, long *speaker_id, float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  try {
    const char *inputs[] = {"phoneme_list", "speaker_id"};
    const char *outputs[] = {"phoneme_length"};
    const std::array<int64_t, 1> phoneme_shape{length};
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);

    std::array<Ort::Value, 2> input_tensors = {ToTensor((int64_t *)phoneme_list, phoneme_shape),
                                               ToTensor(&speaker_id_ll, speaker_shape)};
    Ort::Value output_tensor = ToTensor(output, phoneme_shape);

    status->yukarin_s->Run(Ort::RunOptions{nullptr}, inputs, input_tensors.data(), input_tensors.size(), outputs,
                           &output_tensor, 1);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  return true;
}

bool yukarin_sa_forward(int length, long *vowel_phoneme_list, long *consonant_phoneme_list, long *start_accent_list,
                        long *end_accent_list, long *start_accent_phrase_list, long *end_accent_phrase_list,
                        long *speaker_id, float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  try {
    const char *inputs[] = {
        "length",          "vowel_phoneme_list",       "consonant_phoneme_list", "start_accent_list",
        "end_accent_list", "start_accent_phrase_list", "end_accent_phrase_list", "speaker_id"};
    const char *outputs[] = {"f0_list"};
    const std::array<int64_t, 1> phoneme_shape{length};
    int64_t length_ll = static_cast<int64_t>(length);
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);

    std::array<Ort::Value, 8> input_tensors = {ToTensor(&length_ll, scalar_shape),
                                               ToTensor((int64_t *)vowel_phoneme_list, phoneme_shape),
                                               ToTensor((int64_t *)consonant_phoneme_list, phoneme_shape),
                                               ToTensor((int64_t *)start_accent_list, phoneme_shape),
                                               ToTensor((int64_t *)end_accent_list, phoneme_shape),
                                               ToTensor((int64_t *)start_accent_phrase_list, phoneme_shape),
                                               ToTensor((int64_t *)end_accent_phrase_list, phoneme_shape),
                                               ToTensor(&speaker_id_ll, speaker_shape)};
    Ort::Value output_tensor = ToTensor(output, phoneme_shape);

    status->yukarin_sa->Run(Ort::RunOptions{nullptr}, inputs, input_tensors.data(), input_tensors.size(), outputs,
                            &output_tensor, 1);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }
  return true;
}

bool decode_forward(int length, int phoneme_size, float *f0, float *phoneme, long *speaker_id, float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  try {
    const char *inputs[] = {"f0", "phoneme", "speaker_id"};
    const char *outputs[] = {"wave"};
    const std::array<int64_t, 1> wave_shape{length * 256};
    const std::array<int64_t, 2> f0_shape{length, 1}, phoneme_shape{length, phoneme_size};
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);

    std::array<Ort::Value, 3> input_tensor = {ToTensor(f0, f0_shape), ToTensor(phoneme, phoneme_shape),
                                              ToTensor(&speaker_id_ll, speaker_shape)};
    Ort::Value output_tensor = ToTensor(output, wave_shape);

    status->decode->Run(Ort::RunOptions{nullptr}, inputs, input_tensor.data(), input_tensor.size(), outputs,
                        &output_tensor, 1);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  return true;
}

const char *last_error_message() { return error_message.c_str(); }