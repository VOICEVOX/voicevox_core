#pragma once

#ifdef _WIN32
#ifdef VOICEVOX_CORE_EXPORTS
#define VOICEVOX_CORE_API __declspec(dllexport)
#else  // VOICEVOX_CORE_EXPORTS
#define VOICEVOX_CORE_API __declspec(dllimport)
#endif  // VOICEVOX_CORE_EXPORTS
#else   // _WIN32
#define VOICEVOX_CORE_API
#endif  // _WIN32

/**
 * @fn
 * 初期化する
 * @brief 音声合成するための初期化を行う。他の関数を正しく実行するには先に初期化が必要
 * @param root_dir_path 必要なファイルがあるディレクトリ。相対パス・絶対パスどちらも指定可能。文字コードはUTF-8
 * @param use_gpu trueならGPU用、falseならCPU用の初期化を行う
 * @param cpu_num_threads 推論に用いるスレッド数を設定する。0の場合論理コア数の半分か、物理コア数が設定される
 * @return 成功したらtrue、失敗したらfalse
 * @detail
 * 何度も実行可能。use_gpuを変更して実行しなおすことも可能。
 * 最後に実行したuse_gpuに従って他の関数が実行される。
 */
extern "C" VOICEVOX_CORE_API bool initialize(const char *root_dir_path, bool use_gpu, int cpu_num_threads = 0);

/**
 * @fn
 * 終了処理を行う
 * @brief 終了処理を行う。以降関数を利用するためには再度初期化を行う必要がある。
 * @detail
 * 何度も実行可能。実行せずにexitしても大抵の場合問題ないが、
 * CUDAを利用している場合これを実行しておかないと例外が起こることがある。
 */
extern "C" VOICEVOX_CORE_API void finalize();

/**
 * @fn
 * メタ情報を取得する
 * @brief 話者名や話者IDのリストを取得する
 * @return メタ情報が格納されたjson形式の文字列
 */
extern "C" VOICEVOX_CORE_API const char *metas();

/**
 * @fn
 * 対応デバイス情報を取得する
 * @brief cpu, cudaのうち、使用可能なデバイス情報を取得する
 * @return 各デバイスが使用可能かどうかをboolで格納したjson形式の文字列
 */
extern "C" VOICEVOX_CORE_API const char *supported_devices();

/**
 * @fn
 * 音素ごとの長さを求める
 * @brief 音素列から、音素ごとの長さを求める
 * @param length 音素列の長さ
 * @param phoneme_list 音素列
 * @param speaker_id 話者番号
 * @return 音素ごとの長さ
 */
extern "C" VOICEVOX_CORE_API bool yukarin_s_forward(int64_t length, int64_t *phoneme_list, int64_t *speaker_id,
                                                    float *output);

/**
 * @fn
 * モーラごとの音高を求める
 * @brief モーラごとの音素列とアクセント情報から、モーラごとの音高を求める
 * @param length モーラ列の長さ
 * @param vowel_phoneme_list 母音の音素列
 * @param consonant_phoneme_list 子音の音素列
 * @param start_accent_list アクセントの開始位置
 * @param end_accent_list アクセントの終了位置
 * @param start_accent_phrase_list アクセント句の開始位置
 * @param end_accent_phrase_list アクセント句の終了位置
 * @param speaker_id 話者番号
 * @return モーラごとの音高
 */
extern "C" VOICEVOX_CORE_API bool yukarin_sa_forward(int64_t length, int64_t *vowel_phoneme_list,
                                                     int64_t *consonant_phoneme_list, int64_t *start_accent_list,
                                                     int64_t *end_accent_list, int64_t *start_accent_phrase_list,
                                                     int64_t *end_accent_phrase_list, int64_t *speaker_id,
                                                     float *output);

/**
 * @fn
 * 波形を求める
 * @brief フレームごとの音素と音高から、波形を求める
 * @param length フレームの長さ
 * @param phoneme_size 音素の種類数
 * @param f0 フレームごとの音高
 * @param phoneme フレームごとの音素
 * @param speaker_id 話者番号
 * @return 音声波形
 */
extern "C" VOICEVOX_CORE_API bool decode_forward(int64_t length, int64_t phoneme_size, float *f0, float *phoneme,
                                                 int64_t *speaker_id, float *output);

/**
 * @fn
 * 最後に発生したエラーのメッセージを取得する
 * @return エラーメッセージ
 */
extern "C" VOICEVOX_CORE_API const char *last_error_message();
