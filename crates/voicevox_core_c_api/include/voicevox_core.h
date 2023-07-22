/**
 * @file voicevox_core.h
 *
 * 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。
 *
 * <dl>
 *   <dt id="voicevox-core-safety">
 *     <a href="#voicevox-core-safety">⚠️ Safety</a>
 *   </dt>
 *
 *   <dd>
 * このライブラリの利用にあたっては、いくつかの不変条件が守られている必要がある。本ドキュメントではこの不変条件を<b>安全性要件</b>(_safety
 * requirements_)と呼び、"Safety"というセクションの下に安全性要件を示す。
 *
 * 安全性要件の違反は[Rust言語における<b>未定義動作</b>(_undefined behavior_; 通称<em>UB</em>)](
 * https://doc.rust-lang.org/reference/behavior-considered-undefined.html)を引き起こす。Rustの未定義動作は、Cのそれや[C++のそれ](
 * https://cpprefjp.github.io/implementation-compliance.html#nasal-demon)や[Zigのそれ](
 * https://ziglang.org/documentation/0.10.1/#Undefined-Behavior)などとおおよそ同じであり、引き起こしてはならないものとされる。プログラム全体のどこかに未定義動作が含まれるなら、一般的に、処理系はそれについて何をしてもよい。[変数は同時にtrueでもfalseでもあってもよいし](
 * https://markshroyer.com/2012/06/c-both-true-and-false/)、あなたの鼻から悪魔が飛び出してもよい。このことは通称<b>鼻から悪魔</b>(_nasal
 * demons_)と呼ばれる。
 *
 * 未定義動作はプログラム全体に影響する。<b>運が良ければ</b>セグメンテーション違反などで異常終了するだけだが、ライブラリを呼び出している部分から離れた所で「鼻から悪魔」が起こることもある。そうなったら挙動の予測もデバッグも困難である。これが未定義動作が禁忌とされる所以である。
 *
 * `voicevox_core`全体における安全性要件は以下の通りである。
 *
 * - 「読み込みについて有効」と説明されているポインタは次の条件を満たしていなければならない。
 *     - <b>間接参照可能</b>(_dereferenceable_)である。
 *     - 参照先のメモリは他スレッドから書き込み中ではない。
 * - 「書き込みについて有効」と説明されているポインタは次の条件を満たしていなければならない。
 *     - 参照先のメモリは有効である (ただしメモリアラインメントに沿っている必要は無い)。
 *     - 参照先のメモリは他スレッドからアクセス中ではない。
 * - このライブラリで生成したオブジェクトの解放は、このライブラリが提供するAPIで行わなくてはならない(<code>free</code>や<code>HeapFree</code>で行ってはならない)。
 *
 * 次のことに注意すること。
 *
 * - 次のポインタは読み込みにおいても書き込みにおいても有効ではない。
 *     - ヌルポインタ。
 *     - 解放されたポインタ。
 * - voicevox_coreをアンロードする場合、voicevox_coreが生成したポインタが有効であり続けることは保証されない。
 *   </dd>
 * </dl>
 */

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
 * 参照カウント方式のスマートポインタ(reference-counted smart pointer)であり、
 * ::voicevox_synthesizer_new_with_initialize に渡されるときには参照カウンタがインクリメントされる形でオブジェクトの共有が行われる。
 *
 * \example{
 * ```c
 * OpenJtalkRc *open_jtalk;
 * voicevox_open_jtalk_rc_new("./open_jtalk_dic_utf_8-1.11", &open_jtalk);
 * // ⋮
 * voicevox_open_jtalk_rc_delete(open_jtalk);
 * ```
 * }
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
 * VOICEVOXにおける、ある<b>話者</b>(_speaker_)のある<b>スタイル</b>(_style_)を指す。
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
 * \example{
 * ```c
 * OpenJtalkRc *open_jtalk;
 * voicevox_open_jtalk_rc_new("./open_jtalk_dic_utf_8-1.11", &open_jtalk);
 * ```
 * }
 *
 * \safety{
 * - `open_jtalk_dic_dir`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `out_open_jtalk`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * \example{
 * ```c
 * voicevox_open_jtalk_rc_delete(open_jtalk);
 * ```
 * }
 *
 * \safety{
 * - `open_jtalk`は ::voicevox_open_jtalk_rc_new で得たものでなければならず、また既にこの関数で解放されていてはいけない。
 * - `open_jtalk`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
 * }
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
 * \safety{
 * - `path`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `out_model`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * \safety{
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
 * }
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
 * \safety{
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
 * }
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
 * \safety{
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また既にこの関数で解放されていてはいけない。
 * - `model`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
 * }
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
 * \safety{
 * - `open_jtalk`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_open_jtalk_rc_new で解放されていてはいけない。
 * - `out_synthesizer`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また既にこの関数で解放されていてはいけない。
 * - `synthesizer`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
 * }
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
 * }
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `model_id`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * }
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * }
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `model_id`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * }
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * }
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
 * あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったとしても`cuda`や`dml`は`true`を示しうる。
 *
 * @param [out] output_supported_devices_json サポートデバイス情報のJSON文字列
 *
 * @returns 結果コード
 *
 * \example{
 * ```c
 * char *supported_devices;
 * VoicevoxResultCode result = voicevox_create_supported_devices_json(&supported_devices);
 * ```
 * }
 *
 * \safety{
 * - `output_supported_devices_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_create_supported_devices_json(char **output_supported_devices_json);

/**
 * AudioQueryをJSONとして生成する。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] text UTF-8の日本語テキストまたはAquesTalk形式のkana
 * @param [in] style_id スタイルID
 * @param [in] options オプション
 * @param [out] output_audio_query_json 生成先
 *
 * @returns 結果コード
 *
 * \examples{
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
 * }
 *
 *
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * AccentPhrase (アクセント句)の配列をJSON形式で生成する。
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
 * \examples{
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
 * }
 *
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] accent_phrases_json AccentPhraseの配列のJSON文字列
 * @param [in] style_id スタイルID
 * @param [out] output_accent_phrases_json 生成先
 *
 * @returns 結果コード
 *
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_replace_mora_data(const struct VoicevoxSynthesizer *synthesizer,
                                                          const char *accent_phrases_json,
                                                          VoicevoxStyleId style_id,
                                                          char **output_accent_phrases_json);

/**
 * AccentPhraseの配列の音素長を、特定の声で生成しなおす。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] accent_phrases_json AccentPhraseの配列のJSON文字列
 * @param [in] style_id スタイルID
 * @param [out] output_accent_phrases_json 生成先
 *
 * @returns 結果コード
 *
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
VoicevoxResultCode voicevox_synthesizer_replace_phoneme_length(const struct VoicevoxSynthesizer *synthesizer,
                                                               const char *accent_phrases_json,
                                                               VoicevoxStyleId style_id,
                                                               char **output_accent_phrases_json);

/**
 * AccentPhraseの配列の音高を、特定の声で生成しなおす。
 *
 * 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
 *
 * @param [in] synthesizer 音声シンセサイザ
 * @param [in] accent_phrases_json AccentPhraseの配列のJSON文字列
 * @param [in] style_id スタイルID
 * @param [out] output_accent_phrases_json 生成先
 *
 * @returns 結果コード
 *
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * 生成したWAVデータを解放するには ::voicevox_wav_free を使う。
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `audio_query_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * 生成したWAVデータを解放するには ::voicevox_wav_free を使う。
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
 * \safety{
 * - `synthesizer`は ::voicevox_synthesizer_new_with_initialize で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
 * - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
 * - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
 * }
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
 * \safety{
 * - `json`は以下のAPIで得られたポインタでなくてはいけない。
 *     - ::voicevox_create_supported_devices_json
 *     - ::voicevox_synthesizer_audio_query
 *     - ::voicevox_synthesizer_create_accent_phrases
 *     - ::voicevox_synthesizer_replace_mora_data
 *     - ::voicevox_synthesizer_replace_phoneme_length
 *     - ::voicevox_synthesizer_replace_mora_pitch
 * - 文字列の長さは生成時より変更されていてはならない。
 * - `json`は<a href="#voicevox-core-safety">読み込みと書き込みについて有効</a>でなければならない。
 * - `json`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
 * }
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
void voicevox_json_free(char *json);

/**
 * WAVデータを解放する。
 *
 * @param [in] wav 解放するWAVデータ
 *
 * \safety{
 * - `wav`は以下のAPIで得られたポインタでなくてはいけない。
 *     - ::voicevox_synthesizer_synthesis
 *     - ::voicevox_synthesizer_tts
 * - `wav`は<a href="#voicevox-core-safety">読み込みと書き込みについて有効</a>でなければならない。
 * - `wav`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
 * }
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
 * \examples{
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
 * }
 */
#ifdef _WIN32
__declspec(dllimport)
#endif
const char *voicevox_error_result_to_message(VoicevoxResultCode result_code);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* VOICEVOX_CORE_INCLUDE_GUARD */
