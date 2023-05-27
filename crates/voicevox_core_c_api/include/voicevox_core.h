/// @file

#ifndef VOICEVOX_CORE_INCLUDE_GUARD
#define VOICEVOX_CORE_INCLUDE_GUARD

#ifdef __cplusplus
#include <cstdint>
#else // __cplusplus
#include <stdbool.h>
#include <stdint.h>
#endif // __cplusplus

/**
 * ハードウェアアクセラレーションモードを設定する設定値
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
 * 処理結果を示す結果コード
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
 * 参照カウントで管理されたOpenJtalk
 */
typedef struct OpenJtalkRc OpenJtalkRc;

/**
 * 初期化オプション
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
 * 音声モデル
 */
typedef struct VoicevoxVoiceModel {

} VoicevoxVoiceModel;

/**
 * 音声モデルID
 */
typedef const char *VoicevoxVoiceModelId;

typedef struct VoicevoxSynthesizer {

} VoicevoxSynthesizer;

/**
 * Audio query のオプション
 */
typedef struct VoicevoxAudioQueryOptions {
  /**
   * aquestalk形式のkanaとしてテキストを解釈する
   */
  bool kana;
} VoicevoxAudioQueryOptions;

/**
 * スタイルID
 */
typedef uint32_t VoicevoxStyleId;

/**
 * `accent_phrases` のオプション
 */
typedef struct VoicevoxAccentPhrasesOptions {
  /**
   * aquestalk形式のkanaとしてテキストを解釈する
   */
  bool kana;
} VoicevoxAccentPhrasesOptions;

/**
 * `voicevox_synthesizer_synthesis` のオプション
 */
typedef struct VoicevoxSynthesisOptions {
  /**
   * 疑問文の調整を有効にする
   */
  bool enable_interrogative_upspeak;
} VoicevoxSynthesisOptions;

/**
 * テキスト音声合成オプション
 */
typedef struct VoicevoxTtsOptions {
  /**
   * aquestalk形式のkanaとしてテキストを解釈する
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

/**
 * 参照カウントで管理されたOpenJtalkを生成する
 *
 * # Safety
 * @out_open_jtalk 自動でheap領域が割り当てられるため :voicevox_open_jtalk_rc_delete で開放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_open_jtalk_rc_new(const char *open_jtalk_dic_dir,
                                              struct OpenJtalkRc **out_open_jtalk);

/**
 * 参照カウントで管理されたOpenJtalkを削除する
 * @param [in] open_jtalk 参照カウントで管理されたOpenJtalk
 *
 * # Safety
 * @open_jtalk 有効な :OpenJtalkRc のポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

void voicevox_open_jtalk_rc_delete(struct OpenJtalkRc *open_jtalk);

/**
 * デフォルトの初期化オプションを生成する
 * @return デフォルト値が設定された初期化オプション
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

struct VoicevoxInitializeOptions voicevox_make_default_initialize_options(void);

/**
 * voicevoxのバージョンを取得する
 * @return SemVerでフォーマットされたバージョン
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
 const char *voicevox_get_version(void);

/**
 * vvmファイルパスから音声モデルを生成する
 * @param [in] path vvmファイルパス
 * @param [out] out_model 新しく生成された音声モデルの出力先
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param path null終端文字列であること
 * @param out_model 自動でheapメモリが割り当てられるので ::voicevox_voice_model_delete で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_voice_model_new_from_path(const char *path,
                                                      struct VoicevoxVoiceModel **out_model);

/**
 * 音声モデルのIDを取得する
 * @param [in] model 音声モデル #VoicevoxVoiceModel
 * @return 音声モデルID #VoicevoxVoiceModelId
 *
 * # Safety
 * @param model 有効な #VoicevoxVoiceModel へのポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxVoiceModelId voicevox_voice_model_id(const struct VoicevoxVoiceModel *model);

/**
 * 音声モデルのメタ情報を取得する
 * @param [in] model 音声モデル #VoicevoxVoiceModel
 * @return メタ情報のjson文字列
 *
 * # Safety
 * @param model 有効な #VoicevoxVoiceModel へのポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

const char *voicevox_voice_model_get_metas_json(const struct VoicevoxVoiceModel *model);

/**
 * 音声モデルを破棄する
 * @param [in] model 破棄する音声モデル #VoicevoxVoiceModel
 *
 * # Safety
 * @param model 有効な #VoicevoxVoiceModel へのポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

void voicevox_voice_model_delete(struct VoicevoxVoiceModel *model);

/**
 * 音声シンセサイザを生成して初期化する
 * @param [in] open_jtalk 参照カウントで管理されたOpenJtalk
 * @param [in] options 初期化オプション #VoicevoxInitializeOptions
 * @param [out] out_synthesizer 新しく生成された音声シンセサイザの出力先 #VoicevoxSynthesizer
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param out_synthesizer 自動でheapメモリが割り当てられるので ::voicevox_synthesizer_delete で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_synthesizer_new_with_initialize(const struct OpenJtalkRc *open_jtalk,
                                                            struct VoicevoxInitializeOptions options,
                                                            struct VoicevoxSynthesizer **out_synthesizer);

/**
 * 音声シンセサイザを破棄する
 * @param [in] synthesizer 破棄する音声シンセサイザ #VoicevoxSynthesizer
 *
 * # Safety
 * @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

void voicevox_synthesizer_delete(struct VoicevoxSynthesizer *synthesizer);

/**
 * モデルを読み込む
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] model 音声モデル
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
 * @param model 有効な #VoicevoxVoiceModel へのポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_synthesizer_load_voice_model(struct VoicevoxSynthesizer *synthesizer,
                                                         const struct VoicevoxVoiceModel *model);

/**
 * モデルの読み込みを解除する
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] model_id 音声モデルID
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
 * @param model_id NULL終端文字列であること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_synthesizer_unload_voice_model(struct VoicevoxSynthesizer *synthesizer,
                                                           VoicevoxVoiceModelId model_id);

/**
 * ハードウェアアクセラレーションがGPUモードか判定する
 * @param [in] synthesizer 音声シンセサイザ
 * @return GPUモードならtrue、そうでないならfalse
 *
 * # Safety
 * @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

bool voicevox_synthesizer_is_gpu_mode(const struct VoicevoxSynthesizer *synthesizer);

/**
 * 指定したspeaker_idのモデルが読み込まれているか判定する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] model_id 音声モデルのID #VoicevoxVoiceModelId
 * @return モデルが読み込まれているのであればtrue、そうでないならfalse
 *
 * # Safety
 * @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
 * @param model_id NULL終端文字列
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

bool voicevox_is_loaded_voice_model(const struct VoicevoxSynthesizer *synthesizer,
                                    VoicevoxVoiceModelId model_id);

/**
 * メタ情報をjsonで取得する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @return メタ情報のjson文字列
 *
 * # Safety
 * @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

const char *voicevox_synthesizer_get_metas_json(const struct VoicevoxSynthesizer *synthesizer);

/**
 * サポートデバイス情報をjsonで取得する
 * @param [out] output_supported_devices_json サポートデバイス情報のjson文字列
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param output_supported_devices_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_get_supported_devices_json(char **output_supported_devices_json);

/**
 * デフォルトの AudioQuery のオプションを生成する
 * @return デフォルト値が設定された AudioQuery オプション
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

struct VoicevoxAudioQueryOptions voicevox_make_default_audio_query_options(void);

/**
 * AudioQuery を実行する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] text テキスト。文字コードはUTF-8
 * @param [in] style_id スタイルID #VoicevoxStyleId
 * @param [in] options AudioQueryのオプション #VoicevoxAudioQueryOptions
 * @param [out] output_audio_query_json AudioQuery を json でフォーマットしたもの
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param text null終端文字列であること
 * @param output_audio_query_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
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
 * デフォルトの `accent_phrases` のオプションを生成する
 * @return デフォルト値が設定された `accent_phrases` のオプション
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

struct VoicevoxAccentPhrasesOptions voicevox_make_default_accent_phrases_options(void);

/**
 * create_accent_phrases を実行する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] text テキスト
 * @param [in] style_id スタイルID #VoicevoxStyleId
 * @param [in] output_accent_phrases_json アクセントフレーズのjson文字列
 *
 * # Safety
 * @param text null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
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
 * replace_mora_data を実行する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
 * @param [in] style_id スタイルID #VoicevoxStyleId
 * @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
 *
 * # Safety
 * @param accent_phrases_json null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_synthesizer_replace_mora_data(const struct VoicevoxSynthesizer *synthesizer,
                                                          const char *accent_phrases_json,
                                                          VoicevoxStyleId style_id,
                                                          char **output_accent_phrases_json);

/**
 * replace_phoneme_length を実行する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
 * @param [in] style_id スタイルID #VoicevoxStyleId
 * @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
 *
 * # Safety
 * @param accent_phrases_json null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_synthesizer_replace_phoneme_length(const struct VoicevoxSynthesizer *synthesizer,
                                                               const char *accent_phrases_json,
                                                               VoicevoxStyleId style_id,
                                                               char **output_accent_phrases_json);

/**
 * replace_mora_pitch を実行する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
 * @param [in] style_id スタイルID #VoicevoxStyleId
 * @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
 *
 * # Safety
 * @param accent_phrases_json null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

VoicevoxResultCode voicevox_synthesizer_replace_mora_pitch(const struct VoicevoxSynthesizer *synthesizer,
                                                           const char *accent_phrases_json,
                                                           VoicevoxStyleId style_id,
                                                           char **output_accent_phrases_json);

/**
 * デフォルトの `voicevox_synthesizer_synthesis` のオプションを生成する
 * @return デフォルト値が設定された `voicevox_synthesizer_synthesis` のオプション
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

struct VoicevoxSynthesisOptions voicevox_make_default_synthesis_options(void);

/**
 * AudioQuery から音声合成する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] audio_query_json jsonフォーマットされた AudioQuery
 * @param [in] style_id スタイルID #VoicevoxStyleId
 * @param [in] options AudioQueryから音声合成オプション
 * @param [out] output_wav_length 出力する wav データのサイズ
 * @param [out] output_wav wav データの出力先
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param output_wav_length 出力先の領域が確保された状態でpointerに渡されていること
 * @param output_wav 自動で output_wav_length 分のデータが割り当てられるので ::voicevox_wav_free で解放する必要がある
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
 * デフォルトのテキスト音声合成オプションを生成する
 * @return テキスト音声合成オプション
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

struct VoicevoxTtsOptions voicevox_make_default_tts_options(void);

/**
 * テキスト音声合成を実行する
 * @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
 * @param [in] text テキスト。文字コードはUTF-8
 * @param [in] style_id スタイルID #VoicevoxStyleId
 * @param [in] options テキスト音声合成オプション
 * @param [out] output_wav_length 出力する wav データのサイズ
 * @param [out] output_wav wav データの出力先
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param output_wav_length 出力先の領域が確保された状態でpointerに渡されていること
 * @param output_wav は自動で output_wav_length 分のデータが割り当てられるので ::voicevox_wav_free で解放する必要がある
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
 * jsonフォーマットされたデータのメモリを解放する
 * @param [in] json 解放する json データ
 *
 * # Safety
 * @param voicevox_audio_query で確保されたポインタであり、かつ呼び出し側でバッファの変更を行われていないこと
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

void voicevox_json_free(char *json);

/**
 * wav データのメモリを解放する
 * @param [in] wav 解放する wav データ
 *
 * # Safety
 * @param wav voicevox_tts,voicevox_synthesis で確保されたポインタであり、かつ呼び出し側でバッファの変更を行われていないこと
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

void voicevox_wav_free(uint8_t *wav);

/**
 * エラー結果をメッセージに変換する
 * @param [in] result_code メッセージに変換する result_code
 * @return 結果コードを元に変換されたメッセージ文字列
 */
#ifdef _WIN32
__declspec(dllimport)
#endif

const char *voicevox_error_result_to_message(VoicevoxResultCode result_code);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* VOICEVOX_CORE_INCLUDE_GUARD */
