#include <onnxruntime_cxx_api.h>

#ifdef DIRECTML
#include <dml_provider_factory.h>
#endif

#include <array>
#include <exception>
#include <memory>
#include <optional>
#include <string>
#include <unordered_set>

#include "embedBin/embed.h"
#include "nlohmann/json.hpp"

#ifndef VOICEVOX_CORE_EXPORTS
#define VOICEVOX_CORE_EXPORTS
#endif  // VOICEVOX_CORE_EXPORTS
#include "core.h"

#define NOT_INITIALIZED_ERR "Call initialize() first."
#define NOT_LOADED_ERR "Model is not loaded."
#define ONNX_ERR "ONNX raise exception: "
#define JSON_ERR "JSON parser raise exception: "
#define GPU_NOT_SUPPORTED_ERR "This library is CPU version. GPU is not supported."
#define UNKNOWN_STYLE "Unknown style ID: "

constexpr float PHONEME_LENGTH_MINIMAL = 0.01f;

constexpr std::array<int64_t, 0> scalar_shape{};
constexpr std::array<int64_t, 1> speaker_shape{1};

static std::string error_message;
static bool initialized = false;
static std::string supported_devices_str;

EMBED_DECL(METAS);

namespace EMBED_DECL_NAMESPACE {
EMBED_DECL(YUKARIN_S);
EMBED_DECL(YUKARIN_SA);
EMBED_DECL(DECODE);

/**
 * ３種類のモデルを一纏めにしたもの
 */
struct VVMODEL {
  embed::EMBED_RES (*YUKARIN_S)();
  embed::EMBED_RES (*YUKARIN_SA)();
  embed::EMBED_RES (*DECODE)();
};
const VVMODEL VVMODEL_LIST[] = {
    {YUKARIN_S, YUKARIN_SA, DECODE},
};
}  // namespace EMBED_DECL_NAMESPACE
using EMBED_DECL_NAMESPACE::VVMODEL_LIST;

// 複数モデルある場合のspeaker_idマッピング
// {元のspeaker_id: {モデル番号, 新しいspeaker_id}}
const auto speaker_id_map = std::map<int64_t, std::pair<int64_t, int64_t>>{};

struct SupportedDevices {
  bool cpu = true;
  bool cuda = false;
  bool dml = false;
};
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(SupportedDevices, cpu, cuda, dml);

SupportedDevices get_supported_devices() {
  SupportedDevices devices;
  const auto providers = Ort::GetAvailableProviders();
  for (const std::string &p : providers) {
    if (p == "CUDAExecutionProvider") {
      devices.cuda = true;
    } else if (p == "DmlExecutionProvider") {
      devices.dml = true;
    }
  }
  return devices;
}

struct Status {
  Status(int model_count, bool use_gpu, int cpu_num_threads)
      : memory_info(Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeCPU)) {
    yukarin_s_list = std::vector<std::optional<Ort::Session>>(model_count);
    yukarin_sa_list = std::vector<std::optional<Ort::Session>>(model_count);
    decode_list = std::vector<std::optional<Ort::Session>>(model_count);

    session_options.SetInterOpNumThreads(cpu_num_threads).SetIntraOpNumThreads(cpu_num_threads);
    if (use_gpu) {
#ifdef DIRECTML
      session_options.DisableMemPattern().SetExecutionMode(ExecutionMode::ORT_SEQUENTIAL);
      Ort::ThrowOnError(OrtSessionOptionsAppendExecutionProvider_DML(session_options, 0));
#else
      const OrtCUDAProviderOptions cuda_options;
      session_options.AppendExecutionProvider_CUDA(cuda_options);
#endif
    }
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
  bool load_metas() {
    embed::Resource metas_file = METAS();

    metas = nlohmann::json::parse(metas_file.data, metas_file.data + metas_file.size);
    metas_str = metas.dump();
    supported_styles.clear();
    for (const auto &meta : metas) {
      for (const auto &style : meta["styles"]) {
        supported_styles.insert(style["id"].get<int64_t>());
      }
    }
    return true;
  }

  /**
   * モデルを読み込む
   */
  bool load_model(int model_index) {
    const auto VVMODEL = VVMODEL_LIST[model_index];
    embed::Resource yukarin_s_model = VVMODEL.YUKARIN_S();
    embed::Resource yukarin_sa_model = VVMODEL.YUKARIN_SA();
    embed::Resource decode_model = VVMODEL.DECODE();

    yukarin_s_list[model_index] =
        std::move(Ort::Session(env, yukarin_s_model.data, yukarin_s_model.size, session_options));
    yukarin_sa_list[model_index] =
        std::move(Ort::Session(env, yukarin_sa_model.data, yukarin_sa_model.size, session_options));
    decode_list[model_index] = std::move(Ort::Session(env, decode_model.data, decode_model.size, session_options));
    return true;
  }

  std::string root_dir_path;
  Ort::SessionOptions session_options;
  Ort::MemoryInfo memory_info;

  Ort::Env env{ORT_LOGGING_LEVEL_ERROR};
  std::vector<std::optional<Ort::Session>> yukarin_s_list, yukarin_sa_list, decode_list;

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

/**
 * 複数モデルあった場合のspeaker_idマッピング
 */
std::pair<int64_t, int64_t> get_model_index_and_speaker_id(int64_t speaker_id) {
  const auto found = speaker_id_map.find(speaker_id);
  if (found == speaker_id_map.end()) {
    return {0, speaker_id};
  }
  return found->second;
}

bool initialize(bool use_gpu, int cpu_num_threads, bool load_all_models) {
  initialized = false;

#ifdef DIRECTML
  if (use_gpu && !get_supported_devices().dml) {
#else
  if (use_gpu && !get_supported_devices().cuda) {
#endif /*DIRECTML*/
    error_message = GPU_NOT_SUPPORTED_ERR;
    return false;
  }
  try {
    const int model_count = std::size(VVMODEL_LIST);
    status = std::make_unique<Status>(model_count, use_gpu, cpu_num_threads);
    if (!status->load_metas()) {
      return false;
    }

    if (load_all_models) {
      for (int model_index = 0; model_index < model_count; model_index++) {
        if (!status->load_model(model_index)) {
          return false;
        }
      }

      if (use_gpu) {
        // 一回走らせて十分なGPUメモリを確保させる
        // TODO: 全MODELに対して行う
        int length = 500;
        int phoneme_size = 45;
        std::vector<float> phoneme(length * phoneme_size), f0(length);
        int64_t speaker_id = 0;
        std::vector<float> output(length * 256);
        decode_forward(length, phoneme_size, f0.data(), phoneme.data(), &speaker_id, output.data());
      }
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

bool load_model(int64_t speaker_id) {
  auto [model_index, _] = get_model_index_and_speaker_id(speaker_id);
  return status->load_model(model_index);
}

bool is_model_loaded(int64_t speaker_id) {
  auto [model_index, _] = get_model_index_and_speaker_id(speaker_id);
  return (status->yukarin_s_list[model_index].has_value() && status->yukarin_sa_list[model_index].has_value() &&
          status->decode_list[model_index].has_value());
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
  auto [model_index, model_speaker_id] = get_model_index_and_speaker_id(*speaker_id);
  auto &model = status->yukarin_s_list[model_index];
  if (!model) {
    error_message = NOT_LOADED_ERR;
    return false;
  }
  try {
    const char *inputs[] = {"phoneme_list", "speaker_id"};
    const char *outputs[] = {"phoneme_length"};
    const std::array<int64_t, 1> phoneme_shape{length};

    std::array<Ort::Value, 2> input_tensors = {to_tensor(phoneme_list, phoneme_shape),
                                               to_tensor(&model_speaker_id, speaker_shape)};
    Ort::Value output_tensor = to_tensor(output, phoneme_shape);

    model.value().Run(Ort::RunOptions{nullptr}, inputs, input_tensors.data(), input_tensors.size(), outputs,
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
  auto [model_index, model_speaker_id] = get_model_index_and_speaker_id(*speaker_id);
  auto &model = status->yukarin_sa_list[model_index];
  if (!model) {
    error_message = NOT_LOADED_ERR;
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
                                               to_tensor(&model_speaker_id, speaker_shape)};
    Ort::Value output_tensor = to_tensor(output, phoneme_shape);

    model.value().Run(Ort::RunOptions{nullptr}, inputs, input_tensors.data(), input_tensors.size(), outputs,
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
  auto [model_index, model_speaker_id] = get_model_index_and_speaker_id(*speaker_id);
  auto &model = status->decode_list[model_index];
  if (!model) {
    error_message = NOT_LOADED_ERR;
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
                                              to_tensor(&model_speaker_id, speaker_shape)};

    // TODO: 改善したらここのpadding処理を取り除く
    const auto output_with_padding_size = length_with_padding * 256;
    const std::array<int64_t, 1> wave_shape{output_with_padding_size};

    // TODO: 改善したらここの処理を取り除く
    std::vector<float> output_with_padding(output_with_padding_size, 0.0);
    Ort::Value output_tensor = to_tensor(output_with_padding.data(), wave_shape);

    const char *inputs[] = {"f0", "phoneme", "speaker_id"};
    const char *outputs[] = {"wave"};

    model.value().Run(Ort::RunOptions{nullptr}, inputs, input_tensor.data(), input_tensor.size(), outputs,
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