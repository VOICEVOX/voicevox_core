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
const defaultStyleId = 302;

const log = (text) => self.postMessage({ type: "log", text });

const fetchBytes = async (path) => {
  const response = await fetch(new URL(path, self.location.href));
  if (!response.ok) {
    throw new Error(`${path} fetch failed with HTTP ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
};

const loadResources = async () => {
  Module.FS.mkdir(`/${dictDirName}`);
  await Promise.all(
    dictFiles.map(async (file) => {
      const bytes = await fetchBytes(`./${dictDirName}/${file}`);
      Module.FS.writeFile(`/${dictDirName}/${file}`, bytes);
    }),
  );
  log(`Loaded Open JTalk dictionary (${dictFiles.length} files)`);

  Module.FS.writeFile("/sample.vvm", await fetchBytes("./sample.vvm"));
  log("Loaded sample.vvm");
};

const synthesize = (text, styleId) => {
  const resultCode = Module.ccall(
    "voicevox_browser_synthesize",
    "number",
    ["string", "number"],
    [text, styleId],
  );
  if (resultCode !== 0) {
    throw new Error(`Core synthesis returned ${resultCode}`);
  }
  const wavPointer = Module._voicevox_browser_wav_pointer();
  const wavLength = Module._voicevox_browser_wav_length();
  if (!wavPointer || wavLength < 44) {
    throw new Error("Core returned an invalid WAV buffer");
  }
  const wav = Module.HEAPU8.slice(wavPointer, wavPointer + wavLength);
  const header = new TextDecoder().decode(wav.subarray(0, 4));
  const format = new TextDecoder().decode(wav.subarray(8, 12));
  if (header !== "RIFF" || format !== "WAVE") {
    throw new Error("Core output is not a WAV file");
  }
  return wav;
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

globalThis.Module = {
  noInitialRun: true,
  locateFile: (path, prefix) =>
    new URL(path, prefix || self.location.href).href,
  onRuntimeInitialized: async () => {
    try {
      await loadResources();
      const resultCode = Module._voicevox_browser_initialize();
      if (resultCode !== 0) {
        throw new Error(`Core initialization returned ${resultCode}`);
      }
      // 音声モデルはCore側に読み込み済みなので、MEMFS上のコピーを解放する
      Module.FS.unlink("/sample.vvm");
      ready = true;
      self.postMessage({ type: "ready" });
    } catch (error) {
      self.postMessage({ type: "fatal", message: error.message });
    }
  },
  printErr: (...values) => log(values.join(" ")),
  onAbort: (reason) =>
    self.postMessage({
      type: "fatal",
      message: `Emscripten aborted: ${reason}`,
    }),
};

importScripts("./wasm_browser_api_example.js");
