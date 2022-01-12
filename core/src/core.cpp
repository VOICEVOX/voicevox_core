#include <onnxruntime_cxx_api.h>

#include <array>
#include <cstdlib>
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
static std::string supported_devices_str;

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

struct SupportedDevices {
  bool cpu = true;
  bool cuda = false;
};
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(SupportedDevices, cpu, cuda);

SupportedDevices get_supported_devices() {
  SupportedDevices devices;
  const auto providers = Ort::GetAvailableProviders();
  for (const std::string &p : providers) {
    if (p == "CUDAExecutionProvider") {
      devices.cuda = true;
    }
  }
  return devices;
}

struct Status {
  Status(const char *root_dir_path_utf8, bool use_gpu_)
      : root_dir_path(root_dir_path_utf8),
        use_gpu(use_gpu_),
        memory_info(Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeCPU)),
        yukarin_s(nullptr),
        yukarin_sa(nullptr),
        decode(nullptr) {}

  bool load(int cpu_num_threads) {
    // deprecated in C++20; Use char8_t for utf-8 char in the future.
    fs::path root = fs::u8path(root_dir_path);

    if (!open_metas(root / "metas.json", metas)) {
      return false;
    }
    metas_str = metas.dump();
    supported_styles.clear();
    for (const auto &meta : metas) {
      for (const auto &style : meta["styles"]) {
        supported_styles.insert(style["id"].get<int64_t>());
      }
    }

    std::vector<unsigned char> yukarin_s_model, yukarin_sa_model, decode_model;
    if (!open_models(root / "yukarin_s.onnx", root / "yukarin_sa.onnx", root / "decode.onnx", yukarin_s_model,
                     yukarin_sa_model, decode_model)) {
      return false;
    }
    Ort::SessionOptions session_options;
    session_options.SetInterOpNumThreads(cpu_num_threads).SetIntraOpNumThreads(cpu_num_threads);
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
  std::unordered_set<int64_t> supported_styles;
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

bool validate_speaker_id(int64_t speaker_id) {
  if (status->supported_styles.find(speaker_id) == status->supported_styles.end()) {
    error_message = UNKNOWN_STYLE + std::to_string(speaker_id);
    return false;
  }
  return true;
}

bool initialize(const char *root_dir_path, bool use_gpu, int cpu_num_threads) {
  initialized = false;
  if (use_gpu && !get_supported_devices().cuda) {
    error_message = GPU_NOT_SUPPORTED_ERR;
    return false;
  }
  try {
    status = std::make_unique<Status>(root_dir_path, use_gpu);
    if (!status->load(cpu_num_threads)) {
      return false;
    }
    if (use_gpu) {
      // 一回走らせて十分なGPUメモリを確保させる
      int length = 500;
      int phoneme_size = 45;
      std::vector<float> phoneme(length * phoneme_size), f0(length);
      int64_t speaker_id = 0;
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

const char *supported_devices() {
  SupportedDevices devices = get_supported_devices();
  nlohmann::json json = devices;
  supported_devices_str = json.dump();
  return supported_devices_str.c_str();
}

bool yukarin_s_forward(int64_t length, int64_t *phoneme_list, int64_t *speaker_id, float *output) {
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

    std::array<Ort::Value, 2> input_tensors = {to_tensor(phoneme_list, phoneme_shape),
                                               to_tensor(speaker_id, speaker_shape)};
    Ort::Value output_tensor = to_tensor(output, phoneme_shape);

    status->yukarin_s.Run(Ort::RunOptions{nullptr}, inputs, input_tensors.data(), input_tensors.size(), outputs,
                          &output_tensor, 1);

    for (int64_t i = 0; i < length; i++) {
      if (output[i] < PHONEME_LENGTH_MINIMAL) output[i] = PHONEME_LENGTH_MINIMAL;
    }
  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  return true;
}

bool yukarin_sa_forward(int64_t length, int64_t *vowel_phoneme_list, int64_t *consonant_phoneme_list,
                        int64_t *start_accent_list, int64_t *end_accent_list, int64_t *start_accent_phrase_list,
                        int64_t *end_accent_phrase_list, int64_t *speaker_id, float *output) {
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

    std::array<Ort::Value, 8> input_tensors = {to_tensor(&length, scalar_shape),
                                               to_tensor(vowel_phoneme_list, phoneme_shape),
                                               to_tensor(consonant_phoneme_list, phoneme_shape),
                                               to_tensor(start_accent_list, phoneme_shape),
                                               to_tensor(end_accent_list, phoneme_shape),
                                               to_tensor(start_accent_phrase_list, phoneme_shape),
                                               to_tensor(end_accent_phrase_list, phoneme_shape),
                                               to_tensor(speaker_id, speaker_shape)};
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

std::vector<float> make_f0_with_padding(float *f0, int64_t length, int64_t length_with_padding,
                                        int64_t padding_f0_size) {
  std::vector<float> f0_with_padding;
  f0_with_padding.reserve(length_with_padding);
  f0_with_padding.insert(f0_with_padding.end(), padding_f0_size, 0.0);
  f0_with_padding.insert(f0_with_padding.end(), f0, f0 + length);
  f0_with_padding.insert(f0_with_padding.end(), padding_f0_size, 0.0);
  return f0_with_padding;
}

void insert_padding_phonemes_to_phoneme_with_padding(std::vector<float> &phoneme_with_padding,
                                                     const std::vector<float> &padding_phoneme,
                                                     int64_t padding_phonemes_size) {
  for (auto i = 0; i < padding_phonemes_size; i++) {
    phoneme_with_padding.insert(phoneme_with_padding.end(), padding_phoneme.begin(), padding_phoneme.end());
  }
}

std::vector<float> make_phoneme_with_padding(float *phoneme, int64_t phoneme_size, int64_t length,
                                             int64_t length_with_padding, int64_t padding_phonemes_size) {
  // 無音部分をphonemeに追加するための処理
  // TODO: 改善したらここのcopy処理を取り除く
  std::vector<float> padding_phoneme(phoneme_size, 0.0);
  // 一番はじめのphonemeを有効化することで無音となる
  padding_phoneme[0] = 1;

  std::vector<float> phoneme_with_padding;
  phoneme_with_padding.reserve(length_with_padding * phoneme_size);
  insert_padding_phonemes_to_phoneme_with_padding(phoneme_with_padding, padding_phoneme, padding_phonemes_size);
  const auto phoneme_dimension_size = length * phoneme_size;
  phoneme_with_padding.insert(phoneme_with_padding.end(), phoneme, phoneme + phoneme_dimension_size);
  insert_padding_phonemes_to_phoneme_with_padding(phoneme_with_padding, padding_phoneme, padding_phonemes_size);
  return phoneme_with_padding;
}

void copy_output_with_padding_to_output(const std::vector<float> &output_with_padding, float *output,
                                        int64_t padding_f0_size) {
  const auto padding_sampling_size = padding_f0_size * 256;
  const auto begin_output_copy = output_with_padding.begin() + padding_sampling_size;
  const auto end_output_copy = output_with_padding.end() - padding_sampling_size;
  std::copy(begin_output_copy, end_output_copy, output);
}

bool decode_forward(int64_t length, int64_t phoneme_size, float *f0, float *phoneme, int64_t *speaker_id,
                    float *output) {
  if (!initialized) {
    error_message = NOT_INITIALIZED_ERR;
    return false;
  }
  if (!validate_speaker_id(*speaker_id)) {
    return false;
  }
  try {
    // 音が途切れてしまうのを避けるworkaround処理が入っている
    // TODO: 改善したらここのpadding処理を取り除く
    constexpr auto padding_size = 0.4;
    constexpr auto default_sampling_rate = 24000;
    const auto padding_f0_size =
        static_cast<int64_t>(std::round(static_cast<double>(padding_size * default_sampling_rate) / 256));
    const auto start_and_end_padding_f0_size = 2 * padding_f0_size;
    const auto length_with_padding = length + start_and_end_padding_f0_size;

    // TODO: 改善したらここの処理を取り除く
    auto f0_with_padding = make_f0_with_padding(f0, length, length_with_padding, padding_f0_size);

    // TODO: 改善したらここの処理を取り除く
    const auto padding_phonemes_size = padding_f0_size;
    auto phoneme_with_padding =
        make_phoneme_with_padding(phoneme, phoneme_size, length, length_with_padding, padding_phonemes_size);

    const std::array<int64_t, 2> f0_shape{length_with_padding, 1}, phoneme_shape{length_with_padding, phoneme_size};

    std::array<Ort::Value, 3> input_tensor = {to_tensor(f0_with_padding.data(), f0_shape),
                                              to_tensor(phoneme_with_padding.data(), phoneme_shape),
                                              to_tensor(speaker_id, speaker_shape)};

    // TODO: 改善したらここのpadding処理を取り除く
    const auto output_with_padding_size = length_with_padding * 256;
    const std::array<int64_t, 1> wave_shape{output_with_padding_size};

    // TODO: 改善したらここの処理を取り除く
    std::vector<float> output_with_padding(output_with_padding_size, 0.0);
    Ort::Value output_tensor = to_tensor(output_with_padding.data(), wave_shape);

    const char *inputs[] = {"f0", "phoneme", "speaker_id"};
    const char *outputs[] = {"wave"};

    status->decode.Run(Ort::RunOptions{nullptr}, inputs, input_tensor.data(), input_tensor.size(), outputs,
                       &output_tensor, 1);

    // TODO: 改善したらここのcopy処理を取り除く
    copy_output_with_padding_to_output(output_with_padding, output, padding_f0_size);

  } catch (const Ort::Exception &e) {
    error_message = ONNX_ERR;
    error_message += e.what();
    return false;
  }

  return true;
}

const char *last_error_message() { return error_message.c_str(); }