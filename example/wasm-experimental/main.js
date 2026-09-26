const output = document.querySelector("#output");
const player = document.querySelector("#player");
const form = document.querySelector("#form");
const textInput = document.querySelector("#text");
const styleIdInput = document.querySelector("#style-id");
const submit = document.querySelector("#submit");
const worker = new Worker(new URL("./worker.js", import.meta.url));

const log = (text) => {
  output.textContent += `\n${text}`;
};

const isWav = (wav) =>
  wav.byteLength >= 44 &&
  new TextDecoder().decode(wav.subarray(0, 4)) === "RIFF" &&
  new TextDecoder().decode(wav.subarray(8, 12)) === "WAVE";

const play = (wav) => {
  if (player.src) {
    URL.revokeObjectURL(player.src);
  }
  player.src = URL.createObjectURL(new Blob([wav], { type: "audio/wav" }));
  player.hidden = false;
  player.play().catch(() => {
    log("Autoplay was blocked; press play to listen");
  });
};

form.addEventListener("submit", (event) => {
  event.preventDefault();
  const text = textInput.value;
  if (!text) {
    return;
  }
  submit.disabled = true;
  document.body.dataset.status = "running";
  worker.postMessage({
    type: "synthesize",
    text,
    styleId: Number(styleIdInput.value),
  });
});

worker.addEventListener("message", ({ data }) => {
  if (data.type === "log") {
    log(data.text);
  } else if (data.type === "ready") {
    document.body.dataset.status = "ready";
    log("Core is ready");
    submit.disabled = false;
  } else if (data.type === "success") {
    submit.disabled = false;
    const wav = data.wav instanceof ArrayBuffer ? new Uint8Array(data.wav) : null;
    if (!wav || wav.byteLength !== data.wavLength || !isWav(wav)) {
      document.body.dataset.status = "failed";
      log("Transferred data is incomplete or not a WAV file");
      return;
    }
    document.body.dataset.status = "passed";
    log(`${data.text}; transferred ${wav.byteLength} bytes`);
    play(wav);
  } else if (data.type === "error") {
    submit.disabled = false;
    document.body.dataset.status = "failed";
    log(data.message);
  } else if (data.type === "fatal") {
    submit.disabled = true;
    document.body.dataset.status = "failed";
    log(data.message);
    worker.terminate();
  }
});

worker.addEventListener("error", (event) => {
  submit.disabled = true;
  document.body.dataset.status = "failed";
  log(`Worker error: ${event.message}`);
  worker.terminate();
});
