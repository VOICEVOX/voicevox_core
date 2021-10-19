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
  Status(const char *root_dir_path_, bool use_gpu_)
      : root_dir_path(root_dir_path_),
        use_gpu(use_gpu_),
        memory_info(
            Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeCPU)) {
    fs::path root{root_dir_path};
    Ort::SessionOptions session_options;
#ifdef USE_CUDA
    if (use_gpu) {
      Ort::ThrowOnError(
          OrtSessionOptionsAppendExecutionProvider_CUDA(session_options, 0));
    }
#endif
    yukarin_s = std::make_shared<Ort::Session>(
        env, (root / "yukarin_s.onnx").c_str(), session_options);
    yukarin_sa = std::make_shared<Ort::Session>(
        env, (root / "yukarin_sa.onnx").c_str(), session_options);
    decode = std::make_shared<Ort::Session>(env, (root / "decode.onnx").c_str(),
                                            session_options);
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

const char *metas() { return ""; }

bool yukarin_s_forward(int length, long *phoneme_list, long *speaker_id,
                       float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  try {
    const char *inputs[] = {"phoneme_list", "speaker_id"};
    const char *outputs[] = {"phoneme_length"};
    const std::array<int64_t, 1> phoneme_shape{length};
    std::vector<int64_t> phoneme_list_ll(length);
    std::copy_n(phoneme_list, length, phoneme_list_ll.begin());
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);

    std::array<Ort::Value, 2> input_tensors = {
        Ort::Value::CreateTensor<int64_t>(
            status->memory_info, phoneme_list_ll.data(), length,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(status->memory_info, &speaker_id_ll,
                                          1, speaker_shape.data(),
                                          speaker_shape.size())};
    Ort::Value output_tensor = Ort::Value::CreateTensor<float>(
        status->memory_info, output, length, phoneme_shape.data(),
        phoneme_shape.size());

    status->yukarin_s->Run(Ort::RunOptions{nullptr}, inputs,
                           input_tensors.data(), input_tensors.size(), outputs,
                           &output_tensor, 1);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  return true;
}

bool yukarin_sa_forward(int length, long *vowel_phoneme_list,
                        long *consonant_phoneme_list, long *start_accent_list,
                        long *end_accent_list, long *start_accent_phrase_list,
                        long *end_accent_phrase_list, long *speaker_id,
                        float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  try {
    const char *inputs[] = {"length",
                            "vowel_phoneme_list",
                            "consonant_phoneme_list",
                            "start_accent_list",
                            "end_accent_list",
                            "start_accent_phrase_list",
                            "end_accent_phrase_list",
                            "speaker_id"};
    const char *outputs[] = {"f0_list"};
    const std::array<int64_t, 1> phoneme_shape{length};
    int64_t length_ll = static_cast<int64_t>(length);
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);
    std::vector<int64_t> vowel_phoneme_list_ll(length),
        consonant_phoneme_list_ll(length), start_accent_list_ll(length),
        end_accent_list_ll(length), start_accent_phrase_list_ll(length),
        end_accent_phrase_list_ll(length);
    std::copy_n(vowel_phoneme_list, length, vowel_phoneme_list_ll.begin());
    std::copy_n(consonant_phoneme_list, length,
                consonant_phoneme_list_ll.begin());
    std::copy_n(start_accent_list, length, start_accent_list_ll.begin());
    std::copy_n(end_accent_list, length, end_accent_list_ll.begin());
    std::copy_n(start_accent_phrase_list, length,
                start_accent_phrase_list_ll.begin());
    std::copy_n(end_accent_phrase_list, length,
                end_accent_phrase_list_ll.begin());

    std::array<Ort::Value, 8> input_tensors = {
        Ort::Value::CreateTensor<int64_t>(status->memory_info, &length_ll, 1,
                                          scalar_shape.data(),
                                          scalar_shape.size()),
        Ort::Value::CreateTensor<int64_t>(
            status->memory_info, vowel_phoneme_list_ll.data(), length,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(
            status->memory_info, consonant_phoneme_list_ll.data(), length,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(
            status->memory_info, start_accent_list_ll.data(), length,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(
            status->memory_info, end_accent_list_ll.data(), length,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(
            status->memory_info, start_accent_phrase_list_ll.data(), length,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(
            status->memory_info, end_accent_phrase_list_ll.data(), length,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(status->memory_info, &speaker_id_ll,
                                          1, speaker_shape.data(),
                                          speaker_shape.size())};
    Ort::Value output_tensor = Ort::Value::CreateTensor<float>(
        status->memory_info, output, length, phoneme_shape.data(),
        phoneme_shape.size());

    status->yukarin_sa->Run(Ort::RunOptions{nullptr}, inputs,
                            input_tensors.data(), input_tensors.size(), outputs,
                            &output_tensor, 1);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }
  return true;
}

bool decode_forward(int length, int phoneme_size, float *f0, float *phoneme,
                    long *speaker_id, float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  try {
    const char *inputs[] = {"f0", "phoneme", "speaker_id"};
    const char *outputs[] = {"wave"};
    const std::array<int64_t, 1> wave_shape{length * 256};
    const std::array<int64_t, 2> f0_shape{length, 1},
        phoneme_shape{length, phoneme_size};
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);

    std::array<Ort::Value, 3> input_tensor = {
        Ort::Value::CreateTensor<float>(status->memory_info, f0, length,
                                        f0_shape.data(), f0_shape.size()),
        Ort::Value::CreateTensor<float>(
            status->memory_info, phoneme, length * phoneme_size,
            phoneme_shape.data(), phoneme_shape.size()),
        Ort::Value::CreateTensor<int64_t>(status->memory_info, &speaker_id_ll,
                                          1, speaker_shape.data(),
                                          speaker_shape.size())};
    Ort::Value output_tensor = Ort::Value::CreateTensor<float>(
        status->memory_info, output, length * 256, wave_shape.data(),
        wave_shape.size());

    status->decode->Run(Ort::RunOptions{nullptr}, inputs, input_tensor.data(),
                        input_tensor.size(), outputs, &output_tensor, 1);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  return true;
}

const char *last_error_message() { return error_message.c_str(); }