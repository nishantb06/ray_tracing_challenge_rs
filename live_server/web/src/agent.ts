type Summary = { run_id: string; goal: string; iterations: number; final_verdict: string };
type AgentEvent = { run_id: string; iteration: number; phase: string; payload: unknown };

const runs = document.querySelector<HTMLDivElement>("#agent-runs")!;
const gallery = document.querySelector<HTMLDivElement>("#agent-gallery")!;
const detail = document.querySelector<HTMLElement>("#agent-detail")!;
const status = document.querySelector<HTMLElement>("#agent-status")!;
let source: EventSource | undefined;

async function json(url: string): Promise<any> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

export async function loadRuns(): Promise<void> {
  const data = await json("/agent/runs").catch(() => ({ runs: [] }));
  runs.replaceChildren(...(data.runs as Summary[]).map((run) => {
    const button = document.createElement("button");
    button.textContent = `${run.goal} (${run.final_verdict})`;
    button.onclick = () => void loadRun(run.run_id);
    return button;
  }));
}

async function loadRun(runId: string): Promise<void> {
  source?.close();
  gallery.innerHTML = "";
  const data = await json(`/agent/runs/${encodeURIComponent(runId)}`);
  status.textContent = `Run ${runId}: ${data.summary?.goal ?? "in progress"}`;
  const versions = (data.files as string[]).filter((f) => /^v\d+\.png$/.test(f))
    .map((f) => Number(f.slice(1, 3))).sort((a, b) => a - b);
  versions.forEach((n) => addCard(runId, n));
  source = new EventSource(`/agent/events/${encodeURIComponent(runId)}`);
  source.onmessage = (message) => {
    const event = JSON.parse(message.data) as AgentEvent;
    status.textContent = `Run ${runId}: ${event.phase} (iteration ${event.iteration})`;
    if (event.phase === "RenderCompleted") addCard(runId, event.iteration);
  };
}

function addCard(runId: string, n: number): void {
  if (gallery.querySelector(`[data-v="${n}"]`)) return;
  const button = document.createElement("button");
  button.className = "agent-card";
  button.dataset.v = String(n);
  button.innerHTML = `<strong>v${String(n).padStart(2, "0")}</strong><img alt="Render v${n}" src="/agent/runs/${encodeURIComponent(runId)}/file?kind=png&n=${n}">`;
  button.onclick = () => void showVersion(runId, n);
  gallery.append(button);
}

async function showVersion(runId: string, n: number): Promise<void> {
  const feedback = await fetch(`/agent/runs/${encodeURIComponent(runId)}/file?kind=feedback&n=${n}`)
    .then((r) => r.ok ? r.json() : null);
  const diff = await fetch(`/agent/runs/${encodeURIComponent(runId)}/file?kind=diff&n=${n}`)
    .then((r) => r.ok ? r.text() : "");
  detail.innerHTML = `<h3>v${String(n).padStart(2, "0")}</h3>
    <img class="agent-preview" src="/agent/runs/${encodeURIComponent(runId)}/file?kind=png&n=${n}">
    <h4>Perception feedback</h4><pre>${escapeHtml(JSON.stringify(feedback, null, 2))}</pre>
    <details><summary>Code diff</summary><pre>${escapeHtml(diff)}</pre></details>`;
}
function escapeHtml(value: string): string { const d = document.createElement("div"); d.textContent = value; return d.innerHTML; }

document.querySelector<HTMLButtonElement>("#agent-start")!.onclick = async () => {
  const goal = document.querySelector<HTMLTextAreaElement>("#agent-goal")!.value;
  const mode = document.querySelector<HTMLSelectElement>("#agent-mode")!.value;
  status.textContent = "Starting composer…";
  const response = await fetch("/agent/run", {
    method: "POST", headers: { "content-type": "application/json" },
    body: JSON.stringify({ goal, mode, max_iterations: 25 }),
  });
  const data = await response.json();
  if (!response.ok) { status.textContent = data.error ?? "Unable to start composer"; return; }
  status.textContent = `Run ${data.run_id} started`;
  await loadRun(data.run_id);
};
