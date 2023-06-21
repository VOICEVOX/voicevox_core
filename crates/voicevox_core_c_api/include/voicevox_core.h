/// @file

#ifndef VOICEVOX_CORE_INCLUDE_GUARD
#define VOICEVOX_CORE_INCLUDE_GUARD

/* Generated with cbindgen:0.24.3 */

#ifdef __cplusplus
#include <cstdint>
#else // __cplusplus
#include <stdbool.h>
#include <stdint.h>
#endif // __cplusplus

/**
 * ハードウェアアクセラレーションモードを設定する設定値。
 */
enum VoicevoxAccelerationMode
#ifdef __cplusplus
  : int32_t
#endif // __cplusplus
 {
  /**
   * 実行環境に合った適切なハードウェアアクセラレーションモードを選択する
   */
  VOICEVOX_ACCELERATION_MODE_AUTO = 0,
  /**
   * ハードウェアアクセラレーションモードを"CPU"に設定する
   */
  VOICEVOX_ACCELERATION_MODE_CPU = 1,
  /**
   * ハードウェアアクセラレーションモードを"GPU"に設定する
   */
  VOICEVOX_ACCELERATION_MODE_GPU = 2,
};
#ifndef __cplusplus
typedef int32_t VoicevoxAccelerationMode;
#endif // __cplusplus

/**
 * 処理結果を示す結果コード。
 */
enum VoicevoxResultCode
#ifdef __cplusplus
  : int32_t
#endif // __cplusplus
 {
  /**
   * 成功
   */
  VOICEVOX_RESULT_OK = 0,
  /**
   * open_jtalk辞書ファイルが読み込まれていない
   */
  VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR = 1,
  /**
   * modelの読み込みに失敗した
   */
  VOICEVOX_RESULT_LOAD_MODEL_ERROR = 2,
  /**
   * サポートされているデバイス情報取得に失敗した
   */
  VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR = 3,
  /**
   * GPUモードがサポートされていない
   */
  VOICEVOX_RESULT_GPU_SUPPORT_ERROR = 4,
  /**
   * メタ情報読み込みに失敗した
   */
  VOICEVOX_RESULT_LOAD_METAS_ERROR = 5,
  /**
   * 無効なstyle_idが指定された
   */
  VOICEVOX_RESULT_INVALID_STYLE_ID_ERROR = 6,
  /**
   * 無効なmodel_indexが指定された
   */
  VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR = 7,
  /**
   * 推論に失敗した
   */
  VOICEVOX_RESULT_INFERENCE_ERROR = 8,
  /**
   * コンテキストラベル出力に失敗した
   */
  VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR = 11,
  /**
   * 無効なutf8文字列が入力された
   */
  VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR = 12,
  /**
   * aquestalk形式のテキストの解析に失敗した
   */
  VOICEVOX_RESULT_PARSE_KANA_ERROR = 13,
  /**
   * 無効なAudioQuery
   */
  VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR = 14,
  /**
   * 無効なAccentPhrase
   */
  VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR = 15,
  /**
   * ファイルオープンエラー
   */
  VOICEVOX_OPEN_FILE_ERROR = 16,
  /**
   * Modelを読み込めなかった
   */
  VOICEVOX_VVM_MODEL_READ_ERROR = 17,
  /**
   * すでに読み込まれているModelを読み込もうとした
   */
  VOICEVOX_ALREADY_LOADED_MODEL_ERROR = 18,
  /**
   * Modelが読み込まれていない
   */
  VOICEVOX_UNLOADED_MODEL_ERROR = 19,
};
#ifndef __cplusplus
typedef int32_t VoicevoxResultCode;
#endif // __cplusplus

/**
 * テキスト解析器としてのOpen JTalk。
 *
 * コンストラクトは ::voicevox_open_jtalk_rc_new で行い、デストラクトは ::voicevox_open_jtalk_rc_delete で行う。
 *
 * 参照カウント方式のスマートポインタ (reference-counted smart pointer)
 *
 * ## Example
 *
 * ```c
 * OpenJtalkRc *open_jtalk;
 * voicevox_open_jtalk_rc_new("./open_jtalk_dic_utf_8-1.11", &open_jtalk);
 * voicevox_open_jtalk_rc_delete(open_jtalk);
 * ```
 */
typedef struct OpenJtalkRc OpenJtalkRc;

/**
 * 音声シンセサイザ。
 *
 * コンストラクトは ::voicevox_synthesizer_new_with_initialize で行い、デストラクトは ::voicevox_synthesizer_delete で行う。
 */
typedef struct VoicevoxSynthesizer VoicevoxSynthesizer;

/**
 * 音声モデル。
 *
 * コンストラクトは ::voicevox_voice_model_new_from_path で行い、デストラクトは ::voicevox_voice_model_delete で行う。
 */
typedef struct VoicevoxVoiceModel VoicevoxVoiceModel;

/**
 * 音声モデルID。
 */
typedef const char *VoicevoxVoiceModelId;

/**
 * ::voicevox_synthesizer_new_with_initialize のオプション。
 */
typedef struct VoicevoxInitializeOptions {
  /**
   * ハードウェアアクセラレーションモード
   */
  VoicevoxAccelerationMode acceleration_mode;
  /**
   * CPU利用数を指定
   * 0を指定すると環境に合わせたCPUが利用される
   */
  uint16_t cpu_num_threads;
  /**
   * 全てのモデルを読み込む
   */
  bool load_all_models;
} VoicevoxInitializeOptions;

/**
 * スタイルID。
 *
 * VOICEVOXにおける、ある話者(speaker)のあるスタイル(style) (i.e. 声(voice))を指す。
 */
typedef uint32_t VoicevoxStyleId;

/**
 * ::voicevox_synthesizer_audio_query のオプション。
 */
typedef struct VoicevoxAudioQueryOptions {
  /**
   * aquestalk形式のkanaとしてテキストを解釈する
   */
  bool kana;
} VoicevoxAudioQueryOptions;

/**
 * ::voicevox_synthesizer_create_accent_phrases のオプション。
 */
typedef struct VoicevoxAccentPhrasesOptions {
  /**
   * AquesTalk形式のkanaとしてテキストを解釈する
   */
  bool kana;
} VoicevoxAccentPhrasesOptions;

/**
 * ::voicevox_synthesizer_synthesis のオプション。
 */
typedef struct VoicevoxSynthesisOptions {
  /**
   * 疑問文の調整を有効にする
   */
  bool enable_interrogative_upspeak;
} VoicevoxSynthesisOptions;

/**
 * ::voicevox_synthesizer_tts のオプション。
 */
typedef struct VoicevoxTtsOptions {
  /**
   * AquesTalk形式のkanaとしてテキストを解釈する
   */
  bool kana;
  /**
   * 疑問文の調整を有効にする
   */
  bool enable_interrogative_upspeak;
} VoicevoxTtsOptions;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

extern const struct VoicevoxInitializeOptions voicevox_default_initialize_options;

extern const char *voicevox_version;

extern const struct VoicevoxAudioQueryOptions voicevox_default_audio_query_options;

extern const struct VoicevoxAccentPhrasesOptions voicevox_default_accent_phrases_options;

extern const struct VoicevoxSynthesisOptions voicevox_default_synthesis_options;

extern const struct VoicevoxTtsOptions voicevox_default_tts_options;

/**
 * ::OpenJtalkRc をコンストラクトする。
 *
 * 解放は ::voicevox_open_jtalk_rc_delete で行う。
 *
 * @param [in] open_jtalk_dic_dir 辞書ディレクトリを指すUTF-8のパス
 * @param [out] out_open_jtalk 生成先
 *
 * @returns 結果コード
 *
 * ## Example
 *
 * ```c
 * OpenJtalkRc *open_jtalk;
 * voicevox_open_jtalk_rc_new("./open_jtalk_dic_utf_8-1.11", &open_jtalk);
 * ```
 *
 * ## Safety
 *
 * - `open_jtalk_dic_dir`は有効なヌル終端文字列を指していなければならない。
 * - `out_open_jtalk`はアラインメントに沿っていなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_open_jtalk_rc_new(const char *open_jtalk_dic_dir,
                                              struct OpenJtalkRc **out_open_jtalk);

/**
 * ::OpenJtalkRc をデストラクトする。
 *
 * @param [in] open_jtalk デストラクト対象
 *
 * ## Example
 *
 * ```c
 * voicevox_open_jtalk_rc_delete(open_jtalk);
 * ```
 *
 * ## Safety
 *
 * - `open_jtalk`は ::voicevox_open_jtalk_rc_new で得たものでなければならず、また既にこの関数で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_open_jtalk_rc_delete(struct OpenJtalkRc *open_jtalk);

/**
 * VVMファイルから ::VoicevoxVoiceModel をコンストラクトする。
 *
 * @param [in] path vvmファイルへのUTF-8のファイルパス
 * @param [out] out_model 生成先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `path`は有効なヌル終端文字列を指す。
 * - `out_model`はアラインメントに沿っていなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_voice_model_new_from_path(const char *path,
                                                      struct VoicevoxVoiceModel **out_model);

/**
 * ::VoicevoxVoiceModel からIDを取得する。
 *
 * @param [in] model 音声モデル
 *
 * @returns 音声モデルID
 *
 * ## Safety
 *
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxVoiceModelId voicevox_voice_model_id(const struct VoicevoxVoiceModel *model);

/**
 * ::VoicevoxVoiceModel からメタ情報を取得する。
 *
 * @param [in] model 音声モデル
 *
 * @returns メタ情報のJSON文字列
 *
 * ## Safety
 *
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
const char *voicevox_voice_model_get_metas_json(const struct VoicevoxVoiceModel *model);

/**
 * ::VoicevoxVoiceModel をデストラクトする。
 *
 * @param [in] model 破棄する音声モデル
 *
 * ## Safety
 *
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また既にこの関数で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_voice_model_delete(struct VoicevoxVoiceModel *model);

/**
 * ::VoicevoxSynthesizer をコンストラクトする。
 *
 * @param [in] open_jtalk Open JTalkのオブジェクト
 * @param [in] options オプション
 * @param [out] out_synthesizer 生成先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `open_jtalk`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_open_jtalk_rc_new で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_new_with_initialize(const struct OpenJtalkRc *open_jtalk,
                                                            struct VoicevoxInitializeOptions options,
                                                            struct VoicevoxSynthesizer **out_synthesizer);

/**
 * ::VoicevoxSynthesizer をデストラクトする。
 *
 * @param [in] synthesizer デストラクト対象
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また既にこの関数で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_synthesizer_delete(struct VoicevoxSynthesizer *synthesizer);

/**
 * 音声モデルを読み込む。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] model 音声モデル
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_load_voice_model(struct VoicevoxSynthesizer *synthesizer,
                                                         const struct VoicevoxVoiceModel *model);

/**
 * 音声モデルの読み込みを解除する。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] model_id 音声モデルID
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `model_id`は有効なヌル終端文字列を指していなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_unload_voice_model(struct VoicevoxSynthesizer *synthesizer,
                                                           VoicevoxVoiceModelId model_id);

/**
 * ハードウェアアクセラレーションがGPUモードか判定する。
 *
 * @param [in] synthesizer 音声シンセサイザ
 *
 * @returns GPUモードかどうか
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
bool voicevox_synthesizer_is_gpu_mode(const struct VoicevoxSynthesizer *synthesizer);

/**
 * 指定したIDの音声モデルが読み込まれているか判定する。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] model_id 音声モデルID
 *
 * @returns モデルが読み込まれているかどうか
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `model_id`は有効なヌル終端文字列を指していなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
bool voicevox_synthesizer_is_loaded_voice_model(const struct VoicevoxSynthesizer *synthesizer,
                                                VoicevoxVoiceModelId model_id);

/**
 * 今読み込んでいる音声モデルのメタ情報を、JSONで取得する。
 *
 * @param [in] synthesizer 音声シンセサイザ
 *
 * @return メタ情報のJSON文字列
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
const char *voicevox_synthesizer_get_metas_json(const struct VoicevoxSynthesizer *synthesizer);

/**
 * このライブラリで利用可能なデバイスの情報を、JSONで取得する。
 *
 * JSONの解放は ::voicevox_json_free で行う。
 *
 * @param [out] output_supported_devices_json サポートデバイス情報のJSON文字列
 *
 * @returns 結果コード
 *
 * ## Example
 *
 * ```c
 * char *supported_devices;
 * VoicevoxResultCode result = voicevox_create_supported_devices_json(&supported_devices);
 * ```
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_create_supported_devices_json(char **output_supported_devices_json);

/**
 * AudioQueryをJSONとして生成する。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] text UTF-8の日本語テキストまたはAquesTalk形式のkana
 * @param [in] style_id スタイルID
 * @param [in] options オプション
 * @param [out] output_audio_query_json 生成先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `text`は有効なヌル終端文字列を指していなければならない。
 *
 * ## Examples
 *
 * ```c
 * char *audio_query;
 * voicevox_synthesizer_audio_query(synthesizer,
 *                                  "こんにちは",  // 日本語テキスト
 *                                  2,  // "四国めたん (ノーマル)"
 *                                  (VoicevoxAudioQueryOptions){.kana = false},
 *                                  &audio_query);
 * ```
 *
 * ```c
 * char *audio_query;
 * voicevox_synthesizer_audio_query(synthesizer,
 *                                  "コンニチワ'",  // AquesTalk形式のkana
 *                                  2,  // "四国めたん (ノーマル)"
 *                                  (VoicevoxAudioQueryOptions){.kana = true},
 *                                  &audio_query);
 * ```
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_audio_query(const struct VoicevoxSynthesizer *synthesizer,
                                                    const char *text,
                                                    VoicevoxStyleId style_id,
                                                    struct VoicevoxAudioQueryOptions options,
                                                    char **output_audio_query_json);

/**
 * AccentPhrase (アクセント句)の列をJSON形式で生成する。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] text UTF-8の日本語テキストまたはAquesTalk形式のkana
 * @param [in] style_id スタイルID
 * @param [in] options オプション
 * @param [out] output_accent_phrases_json 生成先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `text`は有効なヌル終端文字列を指していなければならない。
 *
 * ## Examples
 *
 * ```c
 * char *accent_phrases;
 * voicevox_synthesizer_create_accent_phrases(
 *     synthesizer,
 *     "こんにちは",  // 日本語テキスト
 *     2,             // "四国めたん (ノーマル)"
 *     voicevox_default_accent_phrases_options, &accent_phrases);
 * ```
 *
 * ```c
 * char *accent_phrases;
 * voicevox_synthesizer_create_accent_phrases(
 *     synthesizer,
 *     "コンニチワ'",  // AquesTalk形式のkana
 *     2,              // "四国めたん (ノーマル)"
 *     (VoicevoxAccentPhrasesOptions){.kana = true}, &accent_phrases);
 * ```
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_create_accent_phrases(const struct VoicevoxSynthesizer *synthesizer,
                                                              const char *text,
                                                              VoicevoxStyleId style_id,
                                                              struct VoicevoxAccentPhrasesOptions options,
                                                              char **output_accent_phrases_json);

/**
 * AccentPhraseの列の音高・音素長を、特定の声で生成しなおす。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] accent_phrases_json AccentPhraseの列のJSON文字列
 * @param [in] style_id スタイルID
 * @param [out] output_accent_phrases_json 生成先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `accent_phrases_json`は有効なヌル終端文字列を指していなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_replace_mora_data(const struct VoicevoxSynthesizer *synthesizer,
                                                          const char *accent_phrases_json,
                                                          VoicevoxStyleId style_id,
                                                          char **output_accent_phrases_json);

/**
 * AccentPhraseの列の音素長を、特定の声で生成しなおす。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] accent_phrases_json AccentPhraseの列のJSON文字列
 * @param [in] style_id スタイルID
 * @param [out] output_accent_phrases_json 生成先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `accent_phrases_json`は有効なヌル終端文字列を指していなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_replace_phoneme_length(const struct VoicevoxSynthesizer *synthesizer,
                                                               const char *accent_phrases_json,
                                                               VoicevoxStyleId style_id,
                                                               char **output_accent_phrases_json);

/**
 * AccentPhraseの列の音高を、特定の声で生成しなおす。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] accent_phrases_json AccentPhraseの列のJSON文字列
 * @param [in] style_id スタイルID
 * @param [out] output_accent_phrases_json 生成先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `accent_phrases_json`は有効なヌル終端文字列を指していなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_replace_mora_pitch(const struct VoicevoxSynthesizer *synthesizer,
                                                           const char *accent_phrases_json,
                                                           VoicevoxStyleId style_id,
                                                           char **output_accent_phrases_json);

/**
 * AudioQueryから音声合成を行う。
 *
 * 生成したwavを解放するには ::voicevox_wav_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] audio_query_json AudioQueryのJSON文字列
 * @param [in] style_id スタイルID
 * @param [in] options オプション
 * @param [out] output_wav_length 出力のバイト長
 * @param [out] output_wav 出力先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `audio_query_json`は有効なヌル終端文字列を指していなければならない。
 * - `output_wav_length`はアラインメントに沿っていなければならない。
 * - `output_wav`はアラインメントに沿っていなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_synthesis(const struct VoicevoxSynthesizer *synthesizer,
                                                  const char *audio_query_json,
                                                  VoicevoxStyleId style_id,
                                                  struct VoicevoxSynthesisOptions options,
                                                  uintptr_t *output_wav_length,
                                                  uint8_t **output_wav);

/**
 * テキスト音声合成を行う。
 *
 * 生成したwavを解放するには ::voicevox_wav_free を使う。
 *
 * @param [in] synthesizer
 * @param [in] text UTF-8の日本語テキストまたはAquesTalk形式のkana
 * @param [in] style_id スタイルID
 * @param [in] options オプション
 * @param [out] output_wav_length 出力のバイト長
 * @param [out] output_wav 出力先
 *
 * @returns 結果コード
 *
 * ## Safety
 *
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `text`は有効なヌル終端文字列を指していなければならない。
 * - `output_wav_length`はアラインメントに沿っていなければならない。
 * - `output_wav`はアラインメントに沿っていなければならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_tts(const struct VoicevoxSynthesizer *synthesizer,
                                            const char *text,
                                            VoicevoxStyleId style_id,
                                            struct VoicevoxTtsOptions options,
                                            uintptr_t *output_wav_length,
                                            uint8_t **output_wav);

/**
 * JSON文字列を解放する。
 *
 * @param [in] json 解放するJSON文字列
 *
 * ## Safety
 *
 * - `json`は以下のAPIで得られたポインタでなくてはいけない。
 *     -
 * - 文字列の長さは生成時より変更されていてはならない。
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_json_free(char *json);

/**
 * wavデータを解放する。
 *
 * @param [in] wav 解放するwavデータ
 *
 * ## Safety
 *
 * - `wav`は以下のAPIで得られたポインタでなくてはいけない。
 *     - ::voicevox_synthesizer_synthesis
 *     - ::voicevox_synthesizer_tts
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_wav_free(uint8_t *wav);

/**
 * 結果コードに対応したメッセージ文字列を取得する。
 *
 * @param [in] result_code 結果コード
 *
 * @returns 結果コードに対応したメッセージ文字列
 *
 * ## Examples
 *
 * ```c
 * const char *actual = voicevox_error_result_to_message(VOICEVOX_RESULT_OK);
 * const char *EXPECTED = "エラーが発生しませんでした";
 * assert(strcmp(actual, EXPECTED) == 0);
 * ```
 *
 * ```c
 * const char *actual =
 *     voicevox_error_result_to_message(VOICEVOX_RESULT_LOAD_MODEL_ERROR);
 * const char *EXPECTED = "modelデータ読み込みに失敗しました";
 * assert(strcmp(actual, EXPECTED) == 0);
 * ```
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
const char *voicevox_error_result_to_message(VoicevoxResultCode result_code);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* VOICEVOX_CORE_INCLUDE_GUARD */
