#include <onnxruntime_cxx_api.h>

#include <array>
#include <exception>
#include <filesystem>
#include <fstream>
#include <memory>
#include <string>
#include <unordered_set>

#include "nlohmann/json.hpp"

#define VOICEVOX_CORE_EXPORTS
#include "core.h"

#define NOT_INITIALIZED_ERR "Call initialize() first."
#define NOT_FOUND_ERR "No such file or directory: "
#define FAILED_TO_OPEN_MODEL_ERR "Unable to open model files."
#define FAILED_TO_OPEN_METAS_ERR "Unable to open metas.json."
#define ONNX_ERR "ONNX raise exception: "
#define JSON_ERR "JSON parser raise exception: "
#define GPU_NOT_SUPPORTED_ERR "This library is CPU version. GPU is not supported."
#define UNKNOWN_STYLE "Unknown style ID: "

constexpr float PHONEME_LENGTH_MINIMAL = 0.01f;

namespace fs = std::filesystem;
constexpr std::array<int64_t, 0> scalar_shape{};
constexpr std::array<int64_t, 1> speaker_shape{1};

static std::string error_message;
static bool initialized = false;

bool open_models(const fs::path &yukarin_s_path, const fs::path &yukarin_sa_path, const fs::path &decode_path,
                 std::vector<unsigned char> &yukarin_s_model, std::vector<unsigned char> &yukarin_sa_model,
                 std::vector<unsigned char> &decode_model) {
  std::ifstream yukarin_s_file(yukarin_s_path, std::ios::binary), yukarin_sa_file(yukarin_sa_path, std::ios::binary),
      decode_file(decode_path, std::ios::binary);
  if (!yukarin_s_file.is_open() || !yukarin_sa_file.is_open() || !decode_file.is_open()) {
    error_message = FAILED_TO_OPEN_MODEL_ERR;
    return false;
  }

  yukarin_s_model = std::vector<unsigned char>(std::istreambuf_iterator<char>(yukarin_s_file), {});
  yukarin_sa_model = std::vector<unsigned char>(std::istreambuf_iterator<char>(yukarin_sa_file), {});
  decode_model = std::vector<unsigned char>(std::istreambuf_iterator<char>(decode_file), {});
  return true;
}

/**
 * Loads the metas.json.
 *
 * schema:
 * [{
 *  name: string,
 *  styles: [{name: string, id: int}],
 *  speaker_uuid: string,
 *  version: string
 * }]
 */
bool open_metas(const fs::path &metas_path, nlohmann::json &metas) {
  std::ifstream metas_file(metas_path);
  if (!metas_file.is_open()) {
    error_message = FAILED_TO_OPEN_METAS_ERR;
    return false;
  }
  metas_file >> metas;
  return true;
}

bool supports_cuda() {
  const auto providers = Ort::GetAvailableProviders();
  for (const std::string &p : providers) {
    if (p == "CUDAExecutionProvider") {
      return true;
    }
  }

  return false;
}

struct Status {
  Status(const char *root_dir_path_utf8, bool use_gpu_)
      : root_dir_path(root_dir_path_utf8),
        use_gpu(use_gpu_),
        memory_info(Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeCPU)),
        yukarin_s(nullptr),
        yukarin_sa(nullptr),
        decode(nullptr) {}

  bool load() {
    // deprecated in C++20; Use char8_t for utf-8 char in the future.
    fs::path root = fs::u8path(root_dir_path);

    if (!open_metas(root / "metas.json", metas)) {
      return false;
    }
    metas_str = metas.dump();
    supported_styles.clear();
    for (const auto &meta : metas) {
      for (const auto &style : meta["styles"]) {
        supported_styles.insert(style["id"].get<int>());
      }
    }

    std::vector<unsigned char> yukarin_s_model, yukarin_sa_model, decode_model;
    if (!open_models(root / "yukarin_s.onnx", root / "yukarin_sa.onnx", root / "decode.onnx", yukarin_s_model,
                     yukarin_sa_model, decode_model)) {
      return false;
    }
    Ort::SessionOptions session_options;
    yukarin_s = Ort::Session(env, yukarin_s_model.data(), yukarin_s_model.size(), session_options);
    yukarin_sa = Ort::Session(env, yukarin_sa_model.data(), yukarin_sa_model.size(), session_options);
    if (use_gpu) {
      const OrtCUDAProviderOptions cuda_options;
      session_options.AppendExecutionProvider_CUDA(cuda_options);
    }
    decode = Ort::Session(env, decode_model.data(), decode_model.size(), session_options);
    return true;
  }

  std::string root_dir_path;
  bool use_gpu;
  Ort::MemoryInfo memory_info;

  Ort::Env env{ORT_LOGGING_LEVEL_ERROR};
  Ort::Session yukarin_s, yukarin_sa, decode;

  nlohmann::json metas;
  std::string metas_str;
  std::unordered_set<int> supported_styles;
};

static std::unique_ptr<Status> status;

template <typename T, size_t Rank>
Ort::Value to_tensor(T *data, const std::array<int64_t, Rank> &shape) {
  int64_t count = 1;
  for (int64_t dim : shape) {
    count *= dim;
  }
  return Ort::Value::CreateTensor<T>(status->memory_info, data, count, shape.data(), shape.size());
}

bool validate_speaker_id(int speaker_id) {
  if (status->supported_styles.find(speaker_id) == status->supported_styles.end()) {
    error_message = UNKNOWN_STYLE + std::to_string(speaker_id);
    return false;
  }
  return true;
}

bool initialize(const char *root_dir_path, bool use_gpu) {
  initialized = false;
  if (use_gpu && !supports_cuda()) {
    error_message = GPU_NOT_SUPPORTED_ERR;
    return false;
  }
  try {
    status = std::make_unique<Status>(root_dir_path, use_gpu);
    if (!status->load()) {
      return false;
    }
    if (use_gpu) {
      // 一回走らせて十分なGPUメモリを確保させる
      int length = 500;
      int phoneme_size = 45;
      std::vector<float> phoneme(length * phoneme_size), f0(length);
      long speaker_id = 0;
      std::vector<float> output(length * 256);
      decode_forward(length, phoneme_size, f0.data(), phoneme.data(), &speaker_id, output.data());
    }
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  } catch (const nlohmann::json::exception &e) {
    error_message = JSON_ERR;
    error_message += e.what();
    return false;
  } catch (const std::exception &e) {
    error_message = e.what();
    return false;
  }

  initialized = true;
  return true;
}

void finalize() {
  initialized = false;
  status.reset();
}

const char *metas() { return status->metas_str.c_str(); }

bool yukarin_s_forward(int length, long *phoneme_list, long *speaker_id, float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  if (!validate_speaker_id(*speaker_id)) {
    return false;
  }
  try {
    const char *inputs[] = {"phoneme_list", "speaker_id"};
    const char *outputs[] = {"phoneme_length"};
    const std::array<int64_t, 1> phoneme_shape{length};
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);

    std::array<Ort::Value, 2> input_tensors = {to_tensor((int64_t *)phoneme_list, phoneme_shape),
                                               to_tensor(&speaker_id_ll, speaker_shape)};
    Ort::Value output_tensor = to_tensor(output, phoneme_shape);

    status->yukarin_s.Run(Ort::RunOptions{nullptr}, inputs, input_tensors.data(), input_tensors.size(), outputs,
                          &output_tensor, 1);

    for (int i = 0; i < length; i++) {
      if (output[i] < PHONEME_LENGTH_MINIMAL) output[i] = PHONEME_LENGTH_MINIMAL;
    }
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
  if (!validate_speaker_id(*speaker_id)) {
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

    std::array<Ort::Value, 8> input_tensors = {to_tensor(&length_ll, scalar_shape),
                                               to_tensor((int64_t *)vowel_phoneme_list, phoneme_shape),
                                               to_tensor((int64_t *)consonant_phoneme_list, phoneme_shape),
                                               to_tensor((int64_t *)start_accent_list, phoneme_shape),
                                               to_tensor((int64_t *)end_accent_list, phoneme_shape),
                                               to_tensor((int64_t *)start_accent_phrase_list, phoneme_shape),
                                               to_tensor((int64_t *)end_accent_phrase_list, phoneme_shape),
                                               to_tensor(&speaker_id_ll, speaker_shape)};
    Ort::Value output_tensor = to_tensor(output, phoneme_shape);

    status->yukarin_sa.Run(Ort::RunOptions{nullptr}, inputs, input_tensors.data(), input_tensors.size(), outputs,
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
  if (!validate_speaker_id(*speaker_id)) {
    return false;
  }
  try {
    const char *inputs[] = {"f0", "phoneme", "speaker_id"};
    const char *outputs[] = {"wave"};
    const std::array<int64_t, 1> wave_shape{length * 256};
    const std::array<int64_t, 2> f0_shape{length, 1}, phoneme_shape{length, phoneme_size};
    int64_t speaker_id_ll = static_cast<int64_t>(*speaker_id);

    std::array<Ort::Value, 3> input_tensor = {to_tensor(f0, f0_shape), to_tensor(phoneme, phoneme_shape),
                                              to_tensor(&speaker_id_ll, speaker_shape)};
    Ort::Value output_tensor = to_tensor(output, wave_shape);

    status->decode.Run(Ort::RunOptions{nullptr}, inputs, input_tensor.data(), input_tensor.size(), outputs,
                       &output_tensor, 1);
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  return true;
}

const char *last_error_message() { return error_message.c_str(); }