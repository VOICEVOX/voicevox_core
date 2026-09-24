const output = document.querySelector("#output");
const player = document.querySelector("#player");
const worker = new Worker(new URL("./worker.js", import.meta.url));

worker.addEventListener("message", ({ data }) => {
  if (data.type === "log") {
    output.textContent += `\n${data.text}`;
  } else if (data.type === "success") {
    if (!(data.wav instanceof ArrayBuffer)) {
      document.body.dataset.status = "failed";
      output.textContent += "\nWorker did not transfer the WAV buffer";
      worker.terminate();
      return;
    }
    const wav = new Uint8Array(data.wav);
    const header = new TextDecoder().decode(wav.subarray(0, 4));
    const format = new TextDecoder().decode(wav.subarray(8, 12));
    if (
      wav.byteLength !== data.wavLength ||
      wav.byteLength < 44 ||
      header !== "RIFF" ||
      format !== "WAVE"
    ) {
      document.body.dataset.status = "failed";
      output.textContent +=
        "\nTransferred data is incomplete or not a WAV file";
      worker.terminate();
      return;
    }
    document.body.dataset.status = "passed";
    output.textContent += `\n${data.text}; transferred ${wav.byteLength} bytes`;
    player.src = URL.createObjectURL(new Blob([wav], { type: "audio/wav" }));
    player.hidden = false;
    player.play().catch(() => {
      output.textContent += "\nAutoplay was blocked; press play to listen";
    });
    worker.terminate();
  } else if (data.type === "error") {
    document.body.dataset.status = "failed";
    output.textContent += `\n${data.message}`;
    worker.terminate();
  }
});

worker.addEventListener("error", (event) => {
  document.body.dataset.status = "failed";
  output.textContent += `\nWorker error: ${event.message}`;
  worker.terminate();
});
