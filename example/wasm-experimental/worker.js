const successMarker = "BROWSER_SYNTHESIS_OK";

globalThis.Module = {
  noInitialRun: true,
  locateFile: (path, prefix) =>
    new URL(path, prefix || self.location.href).href,
  onRuntimeInitialized: async () => {
    try {
      const modelResponse = await fetch(
        new URL("./sample.vvm", self.location.href),
      );
      if (!modelResponse.ok) {
        throw new Error(`model fetch failed with HTTP ${modelResponse.status}`);
      }
      const modelBytes = new Uint8Array(await modelResponse.arrayBuffer());
      Module.FS.writeFile("/sample.vvm", modelBytes);

      const resultCode = Module._voicevox_browser_synthesize();
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
      if (
        wav.byteLength !== wavLength ||
        header !== "RIFF" ||
        format !== "WAVE"
      ) {
        throw new Error("Core output is not a WAV file");
      }
      self.postMessage(
        {
          type: "success",
          text: `${successMarker}: ${wavLength} bytes`,
          wavLength,
          wav: wav.buffer,
        },
        [wav.buffer],
      );
    } catch (error) {
      self.postMessage({ type: "error", message: error.message });
    }
  },
  printErr: (...values) =>
    self.postMessage({ type: "error", message: values.join(" ") }),
  onAbort: (reason) =>
    self.postMessage({
      type: "error",
      message: `Emscripten aborted: ${reason}`,
    }),
};

importScripts("./wasm_browser_api_example.js");
