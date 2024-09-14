package app

import java.io.File
import jp.hiroshiba.voicevoxcore.*
import kotlinx.cli.*

enum class Mode {
  AUTO,
  CPU,
  GPU
}

fun main(args: Array<String>) {
  val parser = ArgParser("voicevoxcoreexample")
  val mode by parser.option(ArgType.Choice<Mode>(), description = "モード").default(Mode.AUTO)
  val vvmPath by
      parser.option(ArgType.String, fullName = "vvm", description = "vvmファイルへのパス").required()
  val onnxruntime by
      parser
          .option(ArgType.String, description = "ONNX Runtimeのファイル名（モジュール名）もしくはファイルパス")
          .default(Onnxruntime.LIB_VERSIONED_FILENAME)
  val dictDir by
      parser
          .option(ArgType.String, description = "Open JTalkの辞書ディレクトリ")
          .default("./open_jtalk_dic_utf_8-1.11")
  val text by
      parser
          .option(ArgType.String, description = "読み上げさせたい文章")
          .default("この音声は、ボイスボックスを使用して、出力されています。")
  val out by parser.option(ArgType.String, description = "出力wavファイルのパス").default("./output.wav")
  val styleId by parser.option(ArgType.Int, description = "話者IDを指定").default(0)

  parser.parse(args)

  println("Inititalizing: ${mode}, ${onnxruntime}, ${dictDir}")
  val ort = Onnxruntime.loadOnce().filename(onnxruntime).exec()
  val openJtalk = OpenJtalk(dictDir)
  val synthesizer =
      Synthesizer.builder(ort, openJtalk)
          .accelerationMode(
              when (mode) {
                Mode.AUTO -> Synthesizer.AccelerationMode.AUTO
                Mode.CPU -> Synthesizer.AccelerationMode.CPU
                Mode.GPU -> Synthesizer.AccelerationMode.GPU
              }
          )
          .build()

  println("Loading: ${vvmPath}")
  val vvm = VoiceModelFile(vvmPath)
  synthesizer.loadVoiceModel(vvm)

  println("Creating an AudioQuery from the text: ${text}")
  val audioQuery = synthesizer.createAudioQuery(text, styleId)

  println("Synthesizing...")
  val audio = synthesizer.synthesis(audioQuery, styleId).execute()

  println("Saving the audio to ${out}")
  File(out).writeBytes(audio)
}
