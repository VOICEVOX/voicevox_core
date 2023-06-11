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
   * ステータスが初期化されていない
   */
  VOICEVOX_RESULT_UNINITIALIZED_STATUS_ERROR = 6,
  /**
   * 無効なspeaker_idが指定された
   */
  VOICEVOX_RESULT_INVALID_SPEAKER_ID_ERROR = 7,
  /**
   * 無効なmodel_indexが指定された
   */
  VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR = 8,
  /**
   * 推論に失敗した
   */
  VOICEVOX_RESULT_INFERENCE_ERROR = 9,
  /**
   * コンテキストラベル出力に失敗した
   */
  VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR = 10,
  /**
   * 無効なutf8文字列が入力された
   */
  VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR = 11,
  /**
   * aquestalk形式のテキストの解析に失敗した
   */
  VOICEVOX_RESULT_PARSE_KANA_ERROR = 12,
  /**
   * 無効なAudioQuery
   */
  VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR = 13,
  /**
   * 無効なAccentPhrase
   */
  VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR = 14,
};
#ifndef __cplusplus
typedef int32_t VoicevoxResultCode;
#endif // __cplusplus

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
  /**
   * open_jtalkの辞書ディレクトリ
   */
  const char *open_jtalk_dict_dir;
} VoicevoxInitializeOptions;

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
 * `accent_phrases` のオプション
 */
typedef struct VoicevoxAccentPhrasesOptions {
  /**
   * aquestalk形式のkanaとしてテキストを解釈する
   */
  bool kana;
} VoicevoxAccentPhrasesOptions;

/**
 * `voicevox_synthesis` のオプション
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
 * デフォルトの初期化オプションを生成する
 * @return デフォルト値が設定された初期化オプション
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
struct VoicevoxInitializeOptions voicevox_make_default_initialize_options(void);

/**
 * 初期化する
 * @param [in] options 初期化オプション
 * @return 結果コード #VoicevoxResultCode
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_initialize(struct VoicevoxInitializeOptions options);

/**
 * voicevoxのバージョンを取得する
 * @return SemVerでフォーマットされたバージョン
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
const char *voicevox_get_version(void);

/**
 * モデルを読み込む
 * @param [in] speaker_id 読み込むモデルの話者ID
 * @return 結果コード #VoicevoxResultCode
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_load_model(uint32_t speaker_id);

/**
 * ハードウェアアクセラレーションがGPUモードか判定する
 * @return GPUモードならtrue、そうでないならfalse
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
bool voicevox_is_gpu_mode(void);

/**
 * 指定したspeaker_idのモデルが読み込まれているか判定する
 * @return モデルが読み込まれているのであればtrue、そうでないならfalse
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
bool voicevox_is_model_loaded(uint32_t speaker_id);

/**
 * このライブラリの利用を終了し、確保しているリソースを解放する
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_finalize(void);

/**
 * メタ情報をjsonで取得する
 * @return メタ情報のjson文字列
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
const char *voicevox_get_metas_json(void);

/**
 * サポートデバイス情報をjsonで取得する
 * @return サポートデバイス情報のjson文字列
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
const char *voicevox_get_supported_devices_json(void);

/**
 * 音素ごとの長さを推論する
 * @param [in] length phoneme_vector, output のデータ長
 * @param [in] phoneme_vector  音素データ
 * @param [in] speaker_id 話者ID
 * @param [out] output_predict_duration_length 出力データのサイズ
 * @param [out] output_predict_duration_data データの出力先
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param phoneme_vector 必ずlengthの長さだけデータがある状態で渡すこと
 * @param output_predict_duration_data_length uintptr_t 分のメモリ領域が割り当てられていること
 * @param output_predict_duration_data 成功後にメモリ領域が割り当てられるので ::voicevox_predict_duration_data_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_predict_duration(uintptr_t length,
                                             int64_t *phoneme_vector,
                                             uint32_t speaker_id,
                                             uintptr_t *output_predict_duration_data_length,
                                             float **output_predict_duration_data);

/**
 * ::voicevox_predict_durationで出力されたデータを解放する
 * @param[in] predict_duration_data 確保されたメモリ領域
 *
 * # Safety
 * @param predict_duration_data voicevox_predict_durationで確保されたポインタであり、かつ呼び出し側でバッファの変更が行われていないこと
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_predict_duration_data_free(float *predict_duration_data);

/**
 * モーラごとのF0を推論する
 * @param [in] length vowel_phoneme_vector, consonant_phoneme_vector, start_accent_vector, end_accent_vector, start_accent_phrase_vector, end_accent_phrase_vector, output のデータ長
 * @param [in] vowel_phoneme_vector 母音の音素データ
 * @param [in] consonant_phoneme_vector 子音の音素データ
 * @param [in] start_accent_vector アクセントの開始位置のデータ
 * @param [in] end_accent_vector アクセントの終了位置のデータ
 * @param [in] start_accent_phrase_vector アクセント句の開始位置のデータ
 * @param [in] end_accent_phrase_vector アクセント句の終了位置のデータ
 * @param [in] speaker_id 話者ID
 * @param [out] output_predict_intonation_data_length 出力データのサイズ
 * @param [out] output_predict_intonation_data データの出力先
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param vowel_phoneme_vector 必ずlengthの長さだけデータがある状態で渡すこと
 * @param consonant_phoneme_vector 必ずlengthの長さだけデータがある状態で渡すこと
 * @param start_accent_vector 必ずlengthの長さだけデータがある状態で渡すこと
 * @param end_accent_vector 必ずlengthの長さだけデータがある状態で渡すこと
 * @param start_accent_phrase_vector 必ずlengthの長さだけデータがある状態で渡すこと
 * @param end_accent_phrase_vector 必ずlengthの長さだけデータがある状態で渡すこと
 * @param output_predict_intonation_data_length uintptr_t 分のメモリ領域が割り当てられていること
 * @param output_predict_intonation_data 成功後にメモリ領域が割り当てられるので ::voicevox_predict_intonation_data_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_predict_intonation(uintptr_t length,
                                               int64_t *vowel_phoneme_vector,
                                               int64_t *consonant_phoneme_vector,
                                               int64_t *start_accent_vector,
                                               int64_t *end_accent_vector,
                                               int64_t *start_accent_phrase_vector,
                                               int64_t *end_accent_phrase_vector,
                                               uint32_t speaker_id,
                                               uintptr_t *output_predict_intonation_data_length,
                                               float **output_predict_intonation_data);

/**
 * ::voicevox_predict_intonationで出力されたデータを解放する
 * @param[in] predict_intonation_data 確保されたメモリ領域
 *
 * # Safety
 * @param predict_intonation_data 実行後に割り当てられたメモリ領域が解放される
 * @param predict_duration_data voicevox_predict_intonationで確保された，ポインタでありかつ，呼び出し側でバッファの変更を行われていないこと.
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_predict_intonation_data_free(float *predict_intonation_data);

/**
 * decodeを実行する
 * @param [in] length f0 , output のデータ長及び phoneme のデータ長に関連する
 * @param [in] phoneme_size 音素のサイズ phoneme のデータ長に関連する
 * @param [in] f0 基本周波数
 * @param [in] phoneme_vector 音素データ
 * @param [in] speaker_id 話者ID
 * @param [out] output_decode_data_length 出力先データのサイズ
 * @param [out] output_decode_data データ出力先
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param f0 必ず length の長さだけデータがある状態で渡すこと
 * @param phoneme_vector 必ず length * phoneme_size の長さだけデータがある状態で渡すこと
 * @param output_decode_data_length uintptr_t 分のメモリ領域が割り当てられていること
 * @param output_decode_data 成功後にメモリ領域が割り当てられるので ::voicevox_decode_data_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_decode(uintptr_t length,
                                   uintptr_t phoneme_size,
                                   float *f0,
                                   float *phoneme_vector,
                                   uint32_t speaker_id,
                                   uintptr_t *output_decode_data_length,
                                   float **output_decode_data);

/**
 * ::voicevox_decodeで出力されたデータを解放する
 * @param[in] decode_data 確保されたメモリ領域
 *
 * # Safety
 * @param decode_data voicevox_decodeで確保されたポインタであり、かつ呼び出し側でバッファの変更を行われていないこと
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_decode_data_free(float *decode_data);

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
 * @param [in] text テキスト。文字コードはUTF-8
 * @param [in] speaker_id 話者ID
 * @param [in] options AudioQueryのオプション
 * @param [out] output_audio_query_json AudioQuery を json でフォーマットしたもの
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param text null終端文字列であること
 * @param output_audio_query_json 自動でheapメモリが割り当てられるので ::voicevox_audio_query_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_audio_query(const char *text,
                                        uint32_t speaker_id,
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
 * `accent_phrases` を実行する
 * @param [in] text テキスト。文字コードはUTF-8
 * @param [in] speaker_id 話者ID
 * @param [in] options `accent_phrases`のオプション
 * @param [out] output_accent_phrases_json アクセント句の情報の配列を json でフォーマットしたもの
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param text null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_accent_phrases_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_accent_phrases(const char *text,
                                           uint32_t speaker_id,
                                           struct VoicevoxAccentPhrasesOptions options,
                                           char **output_accent_phrases_json);

/**
 *
 * アクセント句の音素長を変更する
 * @param [in] accent_phrases_json アクセント句の配列を json でフォーマットしたもの
 * @param [in] speaker_id 話者ID
 * @param [out] output_accent_phrases_json 音素長が変更されたアクセント句の情報の配列を json でフォーマットしたもの
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param accent_phrases_json null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_accent_phrases_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_mora_length(const char *accent_phrases_json,
                                        uint32_t speaker_id,
                                        char **output_accent_phrases_json);

/**
 * アクセント句の音高を変更する
 * @param [in] accent_phrases_json アクセント句の配列を json でフォーマットしたもの
 * @param [in] speaker_id 話者ID
 * @param [out] output_accent_phrases_json 音高が変更されたアクセント句の情報の配列を json でフォーマットしたもの
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param accent_phrases_json null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_accent_phrases_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_mora_pitch(const char *accent_phrases_json,
                                       uint32_t speaker_id,
                                       char **output_accent_phrases_json);

/**
 * アクセント句の音高・音素長を変更する
 * @param [in] accent_phrases_json アクセント句の配列を json でフォーマットしたもの
 * @param [in] speaker_id 話者ID
 * @param [out] output_accent_phrases_json 音高・音素長が変更されたアクセント句の情報の配列を json でフォーマットしたもの
 * @return 結果コード #VoicevoxResultCode
 *
 * # Safety
 * @param accent_phrases_json null終端文字列であること
 * @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_accent_phrases_json_free で解放する必要がある
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_mora_data(const char *accent_phrases_json,
                                      uint32_t speaker_id,
                                      char **output_accent_phrases_json);

/**
 * デフォルトの `voicevox_synthesis` のオプションを生成する
 * @return デフォルト値が設定された `voicevox_synthesis` のオプション
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
struct VoicevoxSynthesisOptions voicevox_make_default_synthesis_options(void);

/**
 * AudioQuery から音声合成する
 * @param [in] audio_query_json jsonフォーマットされた AudioQuery
 * @param [in] speaker_id  話者ID
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
VoicevoxResultCode voicevox_synthesis(const char *audio_query_json,
                                      uint32_t speaker_id,
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
 * @param [in] text テキスト。文字コードはUTF-8
 * @param [in] speaker_id 話者ID
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
VoicevoxResultCode voicevox_tts(const char *text,
                                uint32_t speaker_id,
                                struct VoicevoxTtsOptions options,
                                uintptr_t *output_wav_length,
                                uint8_t **output_wav);

/**
 * jsonフォーマットされた AudioQuery データのメモリを解放する
 * @param [in] audio_query_json 解放する json フォーマットされた AudioQuery データ
 *
 * # Safety
 * @param voicevox_audio_query で確保されたポインタであり、かつ呼び出し側でバッファの変更を行われていないこと
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_audio_query_json_free(char *audio_query_json);

/**
 * jsonフォーマットされた AccnetPhrase データのメモリを解放する
 * @param [in] accented_phrase_json 解放する json フォーマットされた AccnetPhrase データ
 *
 * # Safety
 * @param voicevox_accent_phrases で確保されたポインタであり、かつ呼び出し側でバッファの変更を行われていないこと
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_accent_phrases_json_free(char *accented_phrase_json);

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
