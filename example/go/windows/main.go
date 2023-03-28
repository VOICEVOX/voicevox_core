package main

import (
	"fmt"
	"os"
	"path/filepath"
	"syscall"
	"unsafe"
)

const COREDLL_DICT_NAME = "voicevox_core.dll"
const OPENJTALK_DICT_NAME = "open_jtalk_dic_utf_8-1.11"

type VoicevoxAccelerationMode int32
type VoicevoxResultCode int32

const (
	VOICEVOX_ACCELERATION_MODE_AUTO VoicevoxAccelerationMode = iota
	VOICEVOX_ACCELERATION_MODE_CPU
	VOICEVOX_ACCELERATION_MODE_GPU
)

const (
	VOICEVOX_RESULT_OK VoicevoxResultCode = iota
	VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR
	VOICEVOX_RESULT_LOAD_MODEL_ERROR
	VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR
	VOICEVOX_RESULT_GPU_SUPPORT_ERROR
	VOICEVOX_RESULT_LOAD_METAS_ERROR
	VOICEVOX_RESULT_UNINITIALIZED_STATUS_ERROR
	VOICEVOX_RESULT_INVALID_SPEAKER_ID_ERROR
	VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR
	VOICEVOX_RESULT_INFERENCE_ERROR
	VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR
	VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR
	VOICEVOX_RESULT_PARSE_KANA_ERROR
	VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR
)

type VoicevoxInitializeOptions struct {
	acceleration_mode   VoicevoxAccelerationMode
	cpu_num_threads     uint16
	load_all_models     bool
	open_jtalk_dict_dir []byte
}

type VoicevoxTtsOptions struct {
	kana                         bool
	enable_interrogative_upspeak bool
}

// デフォルトの初期化オプションを生成する
func voicevox_make_default_initialize_options() VoicevoxInitializeOptions {
	return VoicevoxInitializeOptions{
		acceleration_mode:   VOICEVOX_ACCELERATION_MODE_AUTO,
		cpu_num_threads:     0,
		load_all_models:     false,
		open_jtalk_dict_dir: []byte{},
	}
}

// デフォルトのテキスト音声合成オプションを生成する
func voicevox_make_default_tts_options() VoicevoxTtsOptions {
	return VoicevoxTtsOptions{
		kana:                         true,
		enable_interrogative_upspeak: false,
	}
}

// dllファイルのパスを取得
func GetDllDict() string {
	exePath, err := os.Executable()
	if err != nil {
		panic(err)
	}
	exeDir := filepath.Dir(exePath)
	return filepath.Join(exeDir, COREDLL_DICT_NAME)
}

// OpenJTalk辞書のパスを取得
func GetOpenJTalkDict() string {
	exePath, err := os.Executable()
	if err != nil {
		panic(err)
	}
	exeDir := filepath.Dir(exePath)
	return filepath.Join(exeDir, OPENJTALK_DICT_NAME)
}

// 音声ファイル名取得
func GetWaveFileName() string {
	exePath, err := os.Executable()
	if err != nil {
		panic(err)
	}
	exeDir := filepath.Dir(exePath)
	return filepath.Join(exeDir, "speech.wav")
}

func main() {
	dll := syscall.NewLazyDLL(GetDllDict())
	voicevox_initialize := dll.NewProc("voicevox_initialize")
	voicevox_tts := dll.NewProc("voicevox_tts")
	voicevox_wav_free := dll.NewProc("voicevox_wav_free")
	voicevox_finalize := dll.NewProc("voicevox_finalize")

	fmt.Print("生成する音声の文字列を入力>>")
	speak_words := ""
	fmt.Scan(&speak_words)

	fmt.Println("coreの初期化中")
	initialize_options := voicevox_make_default_initialize_options()
	initialize_options.open_jtalk_dict_dir = []byte(GetOpenJTalkDict())
	initialize_options.load_all_models = true

	result, _, _ := voicevox_initialize.Call(uintptr(unsafe.Pointer(&initialize_options.acceleration_mode)))
	if VoicevoxResultCode(result) != VOICEVOX_RESULT_OK {
		panic(result)
	}

	fmt.Println("音声生成中")
	speaker_id := 0
	var output_binary_size uint
	var output_wav_ptr *uint8
	tts_options := voicevox_make_default_tts_options()
	speak_words_byte := []byte(speak_words)

	result, _, _ = voicevox_tts.Call(uintptr(unsafe.Pointer(&speak_words_byte[0])), uintptr(speaker_id), uintptr(unsafe.Pointer(&tts_options.kana)), uintptr(unsafe.Pointer(&output_binary_size)), uintptr(unsafe.Pointer(&output_wav_ptr)))
	if VoicevoxResultCode(result) != VOICEVOX_RESULT_OK {
		panic(result)
	}
	//出力をポインタから取得
	output_wav := unsafe.Slice(output_wav_ptr, output_binary_size)

	//音声ファイルの保存
	wav_file_path := GetWaveFileName()
	func() {
		f, err := os.Create(wav_file_path)
		if err != nil {
			panic(err)
		}
		//即時関数終了時、ファイルを閉じる
		defer func() {
			if closeErr := f.Close(); closeErr != nil {
				panic(err)
			}
		}()
		f.Write(output_wav)
		fmt.Println(wav_file_path + "に保存されました")
	}() //ここでファイルが閉じる

	fmt.Println("音声データの開放")
	voicevox_wav_free.Call(uintptr(unsafe.Pointer(output_wav_ptr)))

	voicevox_finalize.Call()
}
