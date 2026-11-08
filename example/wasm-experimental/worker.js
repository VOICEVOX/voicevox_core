import createVoicevoxCore from "./wasm_browser_api_example.js";

const dictDirName = "open_jtalk_dic_utf_8-1.11";
const dictFiles = [
  "char.bin",
  "left-id.def",
  "matrix.bin",
  "pos-id.def",
  "rewrite.def",
  "right-id.def",
  "sys.dic",
  "unk.dic",
];
const voiceModelPath = "/sample.vvm";
const defaultStyleId = 302;

const log = (text) => self.postMessage({ type: "log", text });

const fetchBytes = async (path) => {
  const response = await fetch(new URL(path, import.meta.url));
  if (!response.ok) {
    throw new Error(`${path} fetch failed with HTTP ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
};

let core;

const loadCore = () =>
  createVoicevoxCore({
    noInitialRun: true,
    locateFile: (path) => new URL(path, import.meta.url).href,
    printErr: (...values) => log(values.join(" ")),
    onAbort: (reason) =>
      self.postMessage({
        type: "fatal",
        message: `Emscripten aborted: ${reason}`,
      }),
  });

/** Coreの関数を呼び、結果バッファの中身を返す。失敗時は結果バッファのメッセージで例外にする */
const call = (name, argTypes, args) => {
  const resultCode = core.ccall(name, "number", argTypes, args);
  const pointer = core._voicevox_wasm_result_pointer();
  const length = core._voicevox_wasm_result_length();
  const result = core.HEAPU8.slice(pointer, pointer + length);
  if (resultCode !== 0) {
    throw new Error(`${name}: ${new TextDecoder().decode(result)}`);
  }
  return result;
};
const callJson = (name, argTypes, args) =>
  JSON.parse(new TextDecoder().decode(call(name, argTypes, args)));

const initialize = async () => {
  core = await loadCore();
  core.FS.mkdir(`/${dictDirName}`);
  await Promise.all(
    dictFiles.map(async (file) => {
      const bytes = await fetchBytes(`./${dictDirName}/${file}`);
      core.FS.writeFile(`/${dictDirName}/${file}`, bytes);
    }),
  );
  log(`Loaded Open JTalk dictionary (${dictFiles.length} files)`);
  call("voicevox_wasm_initialize", ["string"], [`/${dictDirName}`]);

  core.FS.writeFile(voiceModelPath, await fetchBytes("./sample.vvm"));
  const metas = callJson(
    "voicevox_wasm_load_voice_model",
    ["string"],
    [voiceModelPath],
  );
  // 音声モデルはCore側に読み込み済みなので、MEMFS上のコピーを解放する
  core.FS.unlink(voiceModelPath);
  const styles = metas.flatMap((character) =>
    character.styles.map(
      (style) => `${style.id}: ${character.name} (${style.name})`,
    ),
  );
  log(`Loaded sample.vvm\n${styles.join("\n")}`);
};

const synthesize = (text, styleId) => {
  const audioQuery = callJson(
    "voicevox_wasm_create_audio_query",
    ["string", "number"],
    [text, styleId],
  );
  log(`AudioQuery: ${audioQuery.kana}`);
  return call(
    "voicevox_wasm_synthesis",
    ["string", "number", "number"],
    [JSON.stringify(audioQuery), styleId, 1],
  );
};

let ready = false;

self.addEventListener("message", ({ data }) => {
  if (data.type !== "synthesize") {
    return;
  }
  if (!ready) {
    self.postMessage({ type: "error", message: "Core is not ready yet" });
    return;
  }
  try {
    const styleId = data.styleId ?? defaultStyleId;
    const wav = synthesize(data.text, styleId);
    self.postMessage(
      {
        type: "success",
        text: `Synthesized "${data.text}" (style ${styleId})`,
        wavLength: wav.byteLength,
        wav: wav.buffer,
      },
      [wav.buffer],
    );
  } catch (error) {
    self.postMessage({ type: "error", message: error.message });
  }
});

try {
  await initialize();
  ready = true;
  self.postMessage({ type: "ready" });
} catch (error) {
  self.postMessage({ type: "fatal", message: error.message });
}
