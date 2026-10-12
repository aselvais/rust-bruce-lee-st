const MODE_TITLE = 0;
const MODE_PLAY = 1;
const MODE_PAUSE = 2;
const MODE_CLEAR = 3;
const MODE_OVER = 4;

const KEYS = new Set([
  "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown",
  "KeyA", "KeyD", "KeyW", "KeyS",
  "Space", "Enter", "KeyP", "KeyM", "KeyZ", "KeyX", "KeyJ", "KeyK", "F1",
]);

const statusEl = document.getElementById("status");
const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d", { alpha: false });
const pad = document.getElementById("pad");

const keyHeld = new Set();
const keyEdges = new Set();
const padHeld = new Set();
const padEdges = new Set();

let game = null;
let memory = null;
let image = null;
let speaker = null;
let armed = false;
let last = 0;

class Speaker {
  constructor(audio, clips) {
    this.audio = audio;
    this.clips = clips;
    this.master = audio.createGain();
    this.master.connect(audio.destination);
    this.musicGain = audio.createGain();
    this.musicGain.gain.value = 0.85;
    this.musicGain.connect(this.master);
    this.sfxGain = audio.createGain();
    this.sfxGain.connect(this.master);
    this.track = null;
    this.nodes = [];
    this.paused = false;
    this.levelOffset = 0;
    this.levelStarted = 0;
    this.muted = false;
  }

  toggleMuted() {
    this.muted = !this.muted;
    this.musicGain.gain.value = this.muted ? 0 : 0.85;
  }

  stopMusic() {
    for (const node of this.nodes) {
      try {
        node.stop();
      } catch (_) {
        /* already stopped */
      }
    }
    this.nodes = [];
  }

  startLoop(buffer, offset) {
    const node = this.audio.createBufferSource();
    node.buffer = buffer;
    node.loop = true;
    node.connect(this.musicGain);
    const off = ((offset % buffer.duration) + buffer.duration) % buffer.duration;
    node.start(0, off);
    this.nodes.push(node);
    this.levelStarted = this.audio.currentTime - off;
  }

  startOnce(buffer, when) {
    const node = this.audio.createBufferSource();
    node.buffer = buffer;
    node.connect(this.musicGain);
    node.start(when);
    this.nodes.push(node);
  }

  sync(mode) {
    const track = mode === MODE_TITLE ? "title"
      : mode === MODE_CLEAR ? "clear"
      : mode === MODE_OVER ? "over"
      : "level";

    if (track !== this.track) {
      this.stopMusic();
      this.paused = false;
      this.track = track;
      if (track === "title" || track === "level") {
        this.levelOffset = 0;
        this.startLoop(this.clips[track], 0);
      } else if (track === "clear") {
        const fan = this.clips.clear;
        this.startOnce(fan, 0);
        const bed = this.audio.createBufferSource();
        bed.buffer = this.clips.clearBed;
        bed.loop = true;
        bed.connect(this.musicGain);
        bed.start(this.audio.currentTime + fan.duration);
        this.nodes.push(bed);
      } else {
        this.startOnce(this.clips.over, 0);
      }
    }

    if (track !== "level") {
      return;
    }
    if (mode === MODE_PAUSE && !this.paused) {
      const duration = this.clips.level.duration;
      this.levelOffset = (this.audio.currentTime - this.levelStarted) % duration;
      if (this.levelOffset < 0) {
        this.levelOffset += duration;
      }
      this.stopMusic();
      this.track = "level";
      this.paused = true;
    } else if (mode !== MODE_PAUSE && this.paused) {
      this.startLoop(this.clips.level, this.levelOffset);
      this.paused = false;
    }
  }

  effect(id) {
    if (this.muted) {
      return;
    }
    const buffer = this.clips.fx[id];
    if (!buffer) {
      return;
    }
    const node = this.audio.createBufferSource();
    node.buffer = buffer;
    node.connect(this.sfxGain);
    node.start();
  }
}

function makeBuffer(audio, rate, samples) {
  const buffer = audio.createBuffer(1, samples.length, rate);
  buffer.getChannelData(0).set(samples);
  return buffer;
}

function loadClips(audio, rate) {
  const names = ["title", "level", "clear", "clearBed", "over"];
  const clips = { fx: [] };
  for (let i = 0; i < names.length; i++) {
    clips[names[i]] = makeBuffer(audio, rate, game.clip_samples(i));
  }
  for (let i = 0; i < 10; i++) {
    clips.fx.push(makeBuffer(audio, rate, game.clip_samples(5 + i)));
  }
  return clips;
}

function unlockAudio() {
  armed = true;
  if (!game || speaker) {
    updateStatus();
    return;
  }
  try {
    const audio = new AudioContext({ latencyHint: "interactive" });
    speaker = new Speaker(audio, loadClips(audio, game.sample_rate()));
    audio.resume();
  } catch (err) {
    console.error(err);
  }
  updateStatus();
}

function updateStatus() {
  if (!game) {
    statusEl.textContent = "Loading the game…";
    return;
  }
  if (!armed) {
    statusEl.textContent = "Click the game or press a key for sound. Enter starts.";
    return;
  }
  statusEl.textContent = speaker && speaker.muted
    ? "Muted. Press M to unmute. Enter starts."
    : "Enter starts. P pauses. M mutes.";
}

function readBits() {
  const held = (...codes) => codes.some((code) => keyHeld.has(code));
  const edge = (...codes) => codes.some((code) => keyEdges.has(code));
  if (edge("KeyM") && speaker) {
    speaker.toggleMuted();
    updateStatus();
  }
  let bits = 0;
  if (held("ArrowLeft", "KeyA") || padHeld.has("left")) bits |= 1;
  if (held("ArrowRight", "KeyD") || padHeld.has("right")) bits |= 2;
  if (held("ArrowUp", "KeyW") || padHeld.has("up")) bits |= 4;
  if (held("ArrowDown", "KeyS") || padHeld.has("down")) bits |= 8;
  if (edge("Space") || padEdges.has("jump")) bits |= 16;
  if (held("Space", "ArrowUp", "KeyW") || padHeld.has("jump") || padHeld.has("up")) bits |= 32;
  if (edge("ArrowUp", "KeyW") || padEdges.has("up")) bits |= 64;
  if (edge("KeyZ", "KeyX", "KeyJ", "KeyK") || padEdges.has("attack")) bits |= 128;
  if (edge("Enter") || padEdges.has("start")) bits |= 256;
  if (edge("KeyP") || padEdges.has("pause")) bits |= 512;
  if (edge("F1")) bits |= 1024;
  keyEdges.clear();
  padEdges.clear();
  return bits;
}

function paint() {
  const ptr = game.pixels();
  const len = game.pixel_len();
  const bytes = new Uint8ClampedArray(memory.buffer, ptr, len);
  image.data.set(bytes);
  ctx.putImageData(image, 0, 0);
}

function fit() {
  const header = document.querySelector("header").offsetHeight;
  const footer = document.querySelector("footer").offsetHeight;
  const used = header + pad.offsetHeight + footer + 56;
  const maxW = Math.max(160, window.innerWidth - 24);
  const maxH = Math.max(120, window.innerHeight - used);
  const scale = Math.min(maxW / 640, maxH / 360);
  const integer = Math.floor(scale);
  const chosen = integer >= 1 ? integer : scale;
  canvas.style.width = `${Math.floor(640 * chosen)}px`;
  canvas.style.height = `${Math.floor(360 * chosen)}px`;
}

function frame(now) {
  if (!last) {
    last = now;
  }
  const dt = (now - last) / 1000;
  last = now;
  const mode = game.tick(readBits(), dt);
  canvas.dataset.mode = String(mode);
  paint();
  if (speaker) {
    const cues = game.take_sfx();
    for (let i = 0; i < cues.length; i++) {
      speaker.effect(cues[i]);
    }
    speaker.sync(mode);
  } else {
    game.take_sfx();
  }
  requestAnimationFrame(frame);
}

window.addEventListener("keydown", (event) => {
  if (KEYS.has(event.code)) {
    event.preventDefault();
  }
  unlockAudio();
  if (!event.repeat) {
    keyEdges.add(event.code);
  }
  keyHeld.add(event.code);
});

window.addEventListener("keyup", (event) => {
  keyHeld.delete(event.code);
});

window.addEventListener("blur", () => {
  keyHeld.clear();
  keyEdges.clear();
  padHeld.clear();
  padEdges.clear();
  for (const button of pad.querySelectorAll("button")) {
    button.classList.remove("down");
  }
});

for (const button of pad.querySelectorAll("button")) {
  const name = button.dataset.pad;
  const press = (event) => {
    event.preventDefault();
    unlockAudio();
    button.classList.add("down");
    if (!padHeld.has(name)) {
      padEdges.add(name);
    }
    padHeld.add(name);
  };
  const release = () => {
    padHeld.delete(name);
    button.classList.remove("down");
  };
  button.addEventListener("pointerdown", (event) => {
    try {
      button.setPointerCapture(event.pointerId);
    } catch (_) {
      /* pointer capture is a convenience */
    }
    press(event);
  });
  button.addEventListener("pointerup", release);
  button.addEventListener("pointercancel", release);
  button.addEventListener("contextmenu", (event) => event.preventDefault());
}

canvas.addEventListener("pointerdown", () => {
  unlockAudio();
});

window.addEventListener("resize", fit);

try {
  const wasm = await import("./pkg/bruce_lee.js");
  const output = await wasm.default();
  memory = output.memory;
  game = new wasm.WebGame();
  canvas.width = game.width();
  canvas.height = game.height();
  image = ctx.createImageData(canvas.width, canvas.height);
  fit();
  paint();
  updateStatus();
  requestAnimationFrame(frame);
} catch (err) {
  statusEl.textContent = `The game failed to load: ${err}`;
  console.error(err);
}
