import type {
  ClientStart,
  PixelWire,
  RenderMode,
  ResolutionPreset,
  ResolutionsResponse,
  ScenesResponse,
  ServerMessage,
} from "./protocol";
import { loadRuns } from "./agent";

const renderView = document.getElementById("render-view")!;
const agentView = document.getElementById("agent-view")!;
document.getElementById("tab-render")!.addEventListener("click", () => {
  renderView.hidden = false; agentView.hidden = true;
  document.getElementById("tab-render")!.classList.add("active");
  document.getElementById("tab-agent")!.classList.remove("active");
});
document.getElementById("tab-agent")!.addEventListener("click", () => {
  renderView.hidden = true; agentView.hidden = false;
  document.getElementById("tab-agent")!.classList.add("active");
  document.getElementById("tab-render")!.classList.remove("active");
  void loadRuns();
});

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d")!;
const status = document.getElementById("status") as HTMLSpanElement;
const bar = document.getElementById("bar") as HTMLProgressElement;
const frac = document.getElementById("frac") as HTMLDivElement;
const startBt = document.getElementById("start") as HTMLButtonElement;
const modeSel = document.getElementById("mode") as HTMLSelectElement;
const sceneSel = document.getElementById("scene") as HTMLSelectElement;
const resolutionSel = document.getElementById("resolution") as HTMLSelectElement;

const FALLBACK_SCENES = [
  "group_hexagon",
  "cover_scene",
  "football",
  "single_glass_sphere",
  "reflective_floor",
];

const FALLBACK_RESOLUTIONS: ResolutionPreset[] = [
  { label: "400 × 400", width: 400, height: 400 },
  { label: "300 × 400", width: 300, height: 400 },
  { label: "1200 × 800", width: 1200, height: 800 },
  { label: "1920 × 1280", width: 1920, height: 1280 },
  { label: "3840 × 2160 (4K)", width: 3840, height: 2160 },
];

let ws: WebSocket | null = null;
let width = 0;
let height = 0;
let img: ImageData | null = null;
let done = 0;
let total = 0;

function wsUrl(): string {
  const proto = location.protocol === "https:" ? "wss:" : "ws:";
  return `${proto}//${location.host}/ws`;
}

function setStatus(s: string): void {
  status.textContent = s;
}

function updateProgress(): void {
  if (total > 0) {
    const f = done / total;
    bar.value = f;
    frac.textContent = `${done} / ${total} (${(f * 100).toFixed(1)}%)`;
  }
}

function writePixel(image: ImageData, p: PixelWire, w: number): void {
  const i = (p.y * w + p.x) * 4;
  image.data[i] = p.r;
  image.data[i + 1] = p.g;
  image.data[i + 2] = p.b;
  image.data[i + 3] = 255;
  done += 1;
}

function populateScenes(ids: string[]): void {
  const selected = sceneSel.value || "group_hexagon";
  sceneSel.innerHTML = "";
  for (const id of ids) {
    const opt = document.createElement("option");
    opt.value = id;
    opt.textContent = id;
    if (id === selected) opt.selected = true;
    sceneSel.appendChild(opt);
  }
}

function populateResolutions(presets: ResolutionPreset[]): void {
  const selected = resolutionSel.value || "400x400";
  resolutionSel.innerHTML = "";
  for (const preset of presets) {
    const opt = document.createElement("option");
    opt.value = `${preset.width}x${preset.height}`;
    opt.textContent = preset.label;
    if (opt.value === selected) opt.selected = true;
    resolutionSel.appendChild(opt);
  }
}

function selectedResolution(): { width: number; height: number } {
  const [wStr, hStr] = resolutionSel.value.split("x");
  const w = Number(wStr);
  const h = Number(hStr);
  if (!Number.isFinite(w) || !Number.isFinite(h)) {
    return { width: 400, height: 400 };
  }
  return { width: w, height: h };
}

async function loadScenes(): Promise<void> {
  try {
    const res = await fetch("/scenes");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = (await res.json()) as ScenesResponse;
    if (Array.isArray(data.scenes) && data.scenes.length > 0) {
      populateScenes(data.scenes);
      return;
    }
  } catch {
    // fall through
  }
  populateScenes(FALLBACK_SCENES);
}

async function loadResolutions(): Promise<void> {
  try {
    const res = await fetch("/resolutions");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = (await res.json()) as ResolutionsResponse;
    if (Array.isArray(data.resolutions) && data.resolutions.length > 0) {
      populateResolutions(data.resolutions);
      return;
    }
  } catch {
    // fall through
  }
  populateResolutions(FALLBACK_RESOLUTIONS);
}

function render(): void {
  if (ws) ws.close();
  const { width: reqW, height: reqH } = selectedResolution();
  setStatus(`Connecting… (${reqW}×${reqH})`);
  ws = new WebSocket(wsUrl());

  ws.onopen = () => {
    setStatus(`Connected — rendering ${reqW}×${reqH}…`);
    const msg: ClientStart = {
      type: "Start",
      mode: modeSel.value as RenderMode,
      batch_size: reqW * reqH >= 1_000_000 ? 512 : 128,
      scene: sceneSel.value,
      width: reqW,
      height: reqH,
    };
    ws!.send(JSON.stringify(msg));
  };

  ws.onmessage = (ev: MessageEvent) => {
    const msg = JSON.parse(ev.data as string) as ServerMessage;
    switch (msg.type) {
      case "FrameStart":
        width = msg.width;
        height = msg.height;
        canvas.width = width;
        canvas.height = height;
        img = ctx.createImageData(width, height);
        done = 0;
        total = width * height;
        break;
      case "Pixels":
        if (!img) break;
        for (const p of msg.pixels) writePixel(img, p, width);
        ctx.putImageData(img, 0, 0);
        break;
      case "FrameDone":
        setStatus(`Done (${width}×${height})`);
        break;
    }
    updateProgress();
  };

  ws.onerror = () => setStatus("Connection error");
  ws.onclose = () => {
    if (status.textContent !== "Done" && !status.textContent?.startsWith("Done ")) {
      setStatus("Disconnected");
    }
  };
}

startBt.addEventListener("click", render);
void loadScenes();
void loadResolutions();
