# Ollive Agent — Language-Agnostic Architecture & Porting Specification

> Purpose: a complete, language-agnostic specification of the agentic architecture
> that lives in `agent/`. Target audience: an engineer (or another AI agent) who
> wants to re-implement this system in another language — Rust in particular.
> Any reference implementation detail that is incidental to Python (frameworks,
> module imports, idioms) is described abstractly so the spec transfers cleanly.

---

## 0. How to read this document

- §1 is the bird's-eye view: the core idea (growing DAG + skills) and how data
  flows through one run.
- §2 is the heart of the document: a per-component specification, ordered by
  conceptual dependency. Every component has: **Role**, **Inputs/Outputs**,
  **Public contract**, **Internal behaviour**, **State/Persistence**,
  **Dependencies on other components or services**, **Porting notes**.
- §3 is the gateway contract — the exact HTTP surface a non-Python re-implementer
  must call (with `curl` examples).
- §4 is the MCP server contract and the 11 tools it exposes.
- §5 is how to swap / add / remove tools in the new architecture.
- §6 is the persistence layout ( filesystem schema ) — essential because every
  component writes sidecar JSON / binary files.
- §7 is the prompt-template contract.
- §8 is a concrete Rust porting recipe — crates, modules, types, build steps.
- §9 is a checklist the new implementation can be validated against.

---

## 1. High-Level Architecture

### 1.1 The Core Model

The agent executes a **growing Directed Acyclic Graph (DAG)** of *skill nodes*.
There is no single classic perceive→plan→act loop; the planner **is** the
perceive+plan step and the rest of the graph **is** the act loop, expressed as
data-flow rather than control-flow.

```
            ┌──────────────────────────┐
            │  user message (text)      │
            └─────────────┬────────────┘
                          ▼
                 memory.read(query, history)         ← one probe per run
                          │
                          ▼
            ┌──────────────────────────┐
            │  planner skill node      │   (LLM call → JSON DAG plan)
            └─────────────┬────────────┘
                          │  (graph.extend_from splices the plan into the DAG)
              ┌───────────┴───────────┐
              ▼                       ▼
        [leaf skill]            [leaf skill]   … multiple in parallel
              │                       │
              ▼                       ▼
          (completes)             (completes)
              │                       │
              └───────────┬───────────┘
                          ▼
                  … more nodes (critics auto-injected, recovery replans) …
                          ▼
                     formatter node  → final_answer
```

Invariants the architecture MUST preserve:

1. **Max graph size:** `MAX_NODES = 60`. Once reached the loop terminates and the
   best-available answer is returned.
2. **Planner is tool-blind.** The planner emits **skill names** (from a small
   catalogue), never tool names. Tools belong to skills (see §4, §5).
3. **Memory is read ONCE per run** and the same in-memory `Vec<MemoryItem>` is
   injected into every node's prompt that does not bypass the LLM.
4. **Inputs are references, not values.** A node declares `inputs:` as a list of
   strings: `"USER_QUERY"`, `"n:<id>"`, `"n:<label>"`, `"art:<sha>"`, or a
   literal. Resolution is centralised (see §2.4).
5. **Each completion produces 0+ successor `NodeSpec`s.** Only the planner
   produces them dynamically; other skills are extended with *static* successors
   declared in the config (`internal_successors`).
6. **Critic auto-insertion:** if a skill is flagged `critic: true` in the
  config, every outgoing edge to a freshly-spawned child is rerouted through an
   inserted `critic` node: `src → critic → child`.
7. **Critic-fail recovery is capped ONCE per target node.** A second critic
   failure on the same target stops re-trying that branch and the final answer
   reflects the missing data.
8. **Recovery policy** is centralised (§2.10): transient / validation failures
   are skipped; upstream failures of any non-planner skill trigger a new
   *recovery planner* node feeding the surviving downstream nodes.
9. **The order in which nodes are dispatched is derived from the graph.**
  *Ready nodes* = pending nodes whose every predecessor's status is `complete`
   or `skipped`. Multiple ready nodes run concurrently.

### 1.2 The two co-existing designs

- **Production path** (`flow.py` + `skills.py` + `recovery.py`): the growing DAG
  described above. This is the load-bearing core.
- **Legacy perceive-decide-act single loop** (`perception.py`, `decision.py`,
  `action.py`): present in the codebase but NOT invoked by the production flow.
  The gateway still exposes a hint for them (`auto_route` ∈ `perception |
  memory | decision`); the Rust port may ignore them entirely.

This spec specifies the legacy modules (§2.11/§2.12) for completeness only.

### 1.3 Cross-cutting services

| Service | Lifetime | Backing store |
|---|---|---|
| LLM gateway client (HTTP) | one HTTP client per process | external FastAPI on `:8108` |
| Skill catalogue | static, loaded at startup | YAML config |
| Prompt templates | static, read per node | one `.md` per skill |
| Memory | process | `state/memory.json` (append-only) |
| Vector index | process | `state/index.faiss` + `state/index_ids.json` |
| Artifacts (content-addressable blobs) | process | `state/artifacts/<sha16>.bin` + `.json` |
| Chat transcripts | per chat id | `state/chats/<cid>/{meta,conversation}.json` |
| Graph session | per run | `state/sessions/<run-id>/...` |
| Sandbox | per call | transient tmpdir |
| MCP server | stdio child subprocess, one per tool-allowed skill invocation | — |

---

## 2. Per-Component Specification

Each component below follows the same template:

> **Role** — what it is for.
> **Inputs / Outputs** — public contract.
> **Internal behaviour** — algorithmic detail.
> **State / Persistence** — what files it reads or writes.
> **Dependencies** — on other components and on external services.
> **Porting notes** — implementation-agnostic guidance.

---

### 2.1 `schemas` — typed contracts

**Role:** the canonical, shared data-types of the entire architecture. Every
other component speaks these types. A non-Python implementation MUST define
analogous structs and serialise/deserialise them identically (same JSON
shapes, same field names).

**Public contract (types):**

```
new_id(prefix)       // prefix + ":" + 8 hex chars from a 128-bit random

MemoryKind = "fact" | "preference" | "tool_outcome" | "scratchpad"

MemoryItem {
  id            : string
  kind          : MemoryKind
  keywords      : [string]
  descriptor    : string
  value         : object                 // free-form but NEVER null
  artifact_id   : string | null          // "art:<sha16>" or null
  embedding     : [float] | null         // null for scratchpad or cold-start
  source        : string                 // who/what created it
  run_id        : string                 // owning flow-run
  goal_id       : string | null          // (legacy)
  confidence    : float  = 1.0
  created_at    : datetime (UTC, ISO-8601)
}

Artifact {
  id            : string                 // "art:<sha16>"
  content_type  : string
  size_bytes    : int
  source        : string
  descriptor    : string
  created_at    : datetime
}

ToolCall { name : string, arguments : object }

NodeSpec {                              // what a planner emits per future node
  skill    : string
  inputs   : [string] = []              // references ("USER_QUERY","n:7","art:abc","<literal>")
  metadata : object   = {}              // optional label/question/recovers/failure_report…
}

AgentResult {                           // what executing a node yields
  success     : bool
  agent_name  : string
  output      : object  = {}             // skill-specific (parsed from LLM JSON)
  artifacts   : [string] = []            // "art:<sha16>" handles produced
  successors  : [NodeSpec] = []          // dynamic edges (planner only)
  cost        : float  = 0.0
  elapsed_s   : float  = 0.0
  provider    : string = ""
  error       : string | null
}

NodeState {                             // persisted per node, for resume/replay
  node_id      : string                  // "n:<i>"
  skill        : string
  status       : "pending" | "running" | "complete" | "failed" | "skipped"
  inputs       : [string] = []
  result       : AgentResult | null
  prompt_sent  : string | null
  started_at   : float | null            // epoch seconds
  completed_at : float | null
  retries      : int = 0
}
```

Legacy single-loop types (defined in this module but unused by the DAG path):
`Goal { id, text, done, attach_artifact_id }`,
`Observation { goals: [Goal], all_done, next_unfinished() }`,
`DecisionOutput { answer, tool_call, is_answer }`. The Rust port can skip them.

**State / Persistence:** none; pure types.

**Dependencies:** none.

**Porting notes:** in Rust use `serde`-derivable structs with `Option<T>` where
the spec says "nullable". Keep the JSON field names identical (snake_case).

---

### 2.2 `agent_model` — model profile switching

**Role:** A/B choice of LLM provider+model on a per-call basis. Lets the agent
override what `agent_routing.yaml` on the gateway side would pick.

**Public contract:**

```
profile = ModelProfile {
  name     : string,        // canonical key
  provider : string,        // gateway provider string ("gemini","nvidia",…)
  model    : string | null, // null → use the provider's gateway default
  label    : string         // human-readable
}
resolve(name : string | null) -> ModelProfile       // alias resolution, may raise
set_profile(name : string | null) -> ModelProfile   // sets current profile for the call
get_profile() -> ModelProfile
get_chat_kwargs() -> { provider: string, model?: string }
                      // includes "model" only when profile.model is set
```

**Configured profiles (example):**

| name | provider | model |
|---|---|---|
| `gemini`     (default) | `gemini`  | null |
| `llama-3`    | `nvidia` | `meta/llama-3.1-70b-instruct` |
| `llama-3-8b` | `nvidia` | `meta/llama-3.1-8b-instruct` |

Plus aliases (`g`→`gemini`, `llama3`→`llama-3`, `8b`→`llama-3-8b`, …).

**Internal behaviour:** a thread-/task-local "current profile". `set_profile`
overrides it; every chat call passes through `get_chat_kwargs()` so the
gateway's YAML pinning never silently overrides the agent's choice.

**State / Persistence:** none.

**Dependencies:** none.

**Porting notes:** in Rust use a `thread_local!` or a per-task context (e.g.
`tokio::task_local!`). The agent's chat loop should call `set_profile` at the top
of `handle_turn` and let it propagate through every node.

---

### 2.3 `gateway` — HTTP brain (call the LLM gateway over HTTP)

**Role:** the single bridge to the LLM gateway. Implementations in the original
project auto-launch the gateway subprocess if `GET /v1/routers` does not answer.
A Rust port should **NOT** do this: treat the gateway as an external HTTP
service and document the requirement (e.g. run `cd gateway && uv run main.py`
in a separate terminal).

**Public contract:**

```
ensure_gateway()               // probes /v1/routers; spawn if down (Python only)
embed(text, task_type="retrieval_document")
  -> { embedding, dim, model, provider, latency_ms, attempted }
LLM                            // gateway client class (Python only — DO NOT port)
```

For the Rust port, implement two HTTP helpers:

```rust
async fn chat(body: ChatRequest) -> Result<ChatResponse, GatewayError>;
async fn embed(text: &str, task_type: &str) -> Result<EmbedResponse, GatewayError>;
```

Base URL: `http://localhost:8108` (overridable via env `LLM_GATEWAY_URL`).

**State / Persistence:** none.

**Dependencies:** HTTP client (in Rust: `reqwest`).

**Porting notes:** keep an HTTP keepalive client per process. Always set an
HTTP timeout (≥60 s for chat, ≥10 s for embed). Implement the request body
exactly as in §3.1 to preserve server-side routing and structured-output
behaviour.

---

### 2.4 `skills` — skill registry, prompt renderer, tool catalogue

**Role:** the registry of skills and the per-node dispatch. This is where the
abstract *planner → tools → LLM call* magic actually happens.

#### 2.4.1 Skill catalogue (`agent_config.yaml`)

Each skill is defined by a YAML block. There is NO per-skill code — only the YAML
config plus a Markdown prompt template (§7).

Per-skill fields:

| Field | Type | Meaning |
|---|---|---|
| `prompt` | path | path (relative to agent dir) to the `.md` system-prompt template |
| `description` | string | label for dashboards / replay |
| `tools_allowed` | list[str] | MCP tool names advertised to the model. `[]` = text-only. |
| `provider_pin` | string? | optional preferred provider (overridden by the profile) |
| `internal_successors` | list[str] | static child skills added after this node completes |
| `critic` | bool | if true, insert a `critic` node on every outgoing edge |
| `temperature` | float  | default 0.3 |
| `max_tokens`  | int    | default 2048 |
| `metadata`    | object  | opaque to the orchestrator |

Skill catalogue (the original set):

| Name | tools_allowed | temp | max_tokens | Special |
|---|---|---|---|---|
| `planner`           | `[]`                            | 0.4 | 1500 | decomposes query into DAG; recovers subgraphs |
| `retriever`         | `[search_knowledge]`            | 0.2 | 1200 | searches Memory + FAISS |
| `researcher`        | `[web_search, fetch_url]`      | 0.7 | 2500 | multi-step web research |
| `distiller`         | `[]`                            | 0.1 | 1200 | `critic: true` |
| `summariser`        | `[]`                            | 0.3 | 1200 | |
| `critic`            | `[]`                            | 0.0 |  500 | pass/fail evaluator |
| `formatter`         | `[]`                            | 0.3 | 1500 | TERMINAL; `output.final_answer` returned to user |
| `coder`             | `[]`                            | 0.2 | 1500 | `internal_successors:[sandbox_executor]` |
| `sandbox_executor`  | `[]`                            | 0.0 |  400 | runs coder's code locally |
| `browser`           | `[]`                            | 0.3 | 1500 | reserved stub |

#### 2.4.2 `Skill` and `SkillRegistry`

```
Skill { name, prompt_path, description, tools_allowed, internal_successors,
        critic, provider_pin, temperature, max_tokens }
SkillRegistry.load(path)              // reads YAML; builds name → Skill map
SkillRegistry.get(name) -> Skill      // raises if unknown
Skill.prompt_template() -> string    // reads the .md file (or fallback string)
```

#### 2.4.3 `resolve_inputs` — central reference materialiser

Given a node's `inputs: [string]` list and the current graph node map, resolve
each input to a *materialised* dict for prompt injection:

| Input token | Resolved value |
|---|---|
| `"USER_QUERY"` | `{id:"USER_QUERY", kind:"query", value:<query>}` |
| `"n:<i>"` (a node already in graph) | `{id:"n:<i>", kind:"upstream", skill:<agent_name>, output:<AgentResult.output>}` |
| `"n:<i>"` upstream result missing | `{id, kind:"upstream-missing"}` |
| `"art:<sha16>"` (artifact exists) | `{id, kind:"artifact", text:<utf-8 bytes, capped at 20_000 chars>}` |
| `"art:<sha16>"` missing | `{id, kind:"artifact-missing"}` |
| anything else | `{id, kind:"literal", value:<token>}` |

The 20 KB per-artifact cap is essential: it stops one giant document blowing
the prompt context. Larger blobs live in the artifact store and are summarised
in-place if needed (the model never receives raw >20 KB).

#### 2.4.4 `render_prompt`

Concatenate, in order:

1. `skill.prompt_template().rstrip()`
2. If `skill.name ∈ {"planner","formatter"}`:
   - PERSONA block if a persona is set, but only for these two skills.
   - CHAT-HISTORY block if chat context is set.
3. `USER_QUERY: <query>` ONLY if a `"USER_QUERY"` token is among resolved inputs.
4. `QUESTION: <question>` if `metadata.question` is set.
5. `FAILURE:\n<failure_report>` if a recovery planner is running (set by the
   recovery path).
6. `MEMORY HITS (<n> from FAISS):\n<formatted hits>` if memory hits exist
   (formatted as `[<kind>] <descriptor:200>` + 2000-char chunk/raw preview).
7. `INPUTS:\n<json.dumps(resolved, indent=2)>` — always — capped at 20000 chars.

The final prompt is a single string. It is persisted into the `NodeState.prompt_sent`.

#### 2.4.5 `tool_payload(names)` — the model-visibility filter

Returns `null` if `names` is empty (no tools → plain text chat).
Otherwise returns the list of tool descriptors mapped from a private
`_TOOL_CATALOG` (see §4 for the actual schema). **Only tools present in this
catalogue are visible to the model.** The MCP server may implement many more
tools that the model can't call — they exist for direct CLI / debugging use or
for the legacy single-loop path.

#### 2.4.6 `parse_skill_json(text)` — pull JSON out of an LLM response

1. Strip ``` markdown fences (optional language tag).
2. Try `json.loads`. On failure, locate the outermost `{…}` substring and retry.
3. On failure raise (the calling node becomes a *failure*).

#### 2.4.7 `run_skill` — the dispatch entrypoint

```
async run_skill(skill, node_id, graph_nodes, session_id, query, failure_report,
                *, memory_hits=none, chat_context=none, persona=none)
  -> (AgentResult, prompt_sent_string)
```

Steps:

1. `resolved = resolve_inputs(node_inputs, graph_nodes, query)`
2. `question = node.metadata.question` if any.
3. `rendered = render_prompt(skill, query, resolved, failure_report, memory_hits,
   question, chat_context, persona)`
4. **Sandbox shortcut:** if `skill.name == "sandbox_executor"`:
   - Scan `resolved` for an upstream `coder` node's `output["code"]`.
   - If missing → `AgentResult(success=False, error="no code in upstream coder output")`.
   - Otherwise call `sandbox.run_python(code)` and build `AgentResult(success =
     exit_code==0 && !timed_out, output={…})`. **This path bypasses the LLM
     entirely.**
5. Otherwise (LLM path):
   - `tools = tool_payload(skill.tools_allowed)`
   - `profile = agent_model.get_chat_kwargs()`
   - If `tools is not None`: `reply = mcp_runner.run_with_tools(
     prompt=rendered, tools_payload=tools, agent=skill.name,
     session_id=session_id, max_tokens=skill.max_tokens,
     temperature=skill.temperature, **profile)`
   - Else: `reply = gateway.chat(prompt=rendered, agent=skill.name,
     session=session_id, max_tokens=…, temperature=…, **profile)`
6. `parsed = parse_skill_json(reply.text)` (raises → AgentResult(success=False))
7. **Successors collection:**
   - `successors = parsed.pop("successors", [])` (any skill)
   - If `skill.name == "planner"`: ALSO take successors from `parsed["nodes"]`
     (planner's primary output — the new DAG plan).
   - Each successor MUST validate against `NodeSpec`. Collect validation errors
     into `rejected`.
8. If `rejected` is non-empty → `AgentResult(success=False, error="<skill>: N
   malformed NodeSpec(s)…", output=parsed, successors=successors,
   provider=reply.provider)` (the failure path will surface this).
9. Else → `AgentResult(success=True, agent_name=skill.name, output=parsed,
   successors=successors, elapsed_s=…, provider=reply.provider)`.

**State / Persistence:** reads `agent_config.yaml` and `prompts/<skill>.md`.
Writes nothing itself (the *caller* persists `NodeState`).

**Dependencies:** gateway (chat), mcp_runner (tool loop), sandbox (skill
shortcut), agent_model, schemas, artifacts (for `resolve_inputs`).

**Porting notes:**
- Validation of planner output must reject malformed NodeSpecs eagerly; the
  architecture leans on that to make replay / recovery deterministic.
- A skill that has `tools_allowed` always goes through `run_with_tools`. A
  text-only skill goes through a single `chat` call.

---

### 2.5 `flow` — the DAG orchestrator and CLI

**Role:** the loop that grows the graph, dispatches ready nodes concurrently,
handles critic verdicts and failures, persists state, and returns the final
answer.

**Constants:** `MAX_NODES = 60`.

#### 2.5.1 `Graph`

Wraps a generic directed graph (the original uses `networkx.DiGraph`). Node IDs
are `"n:<counter>"`, starting at `n:1`. Every node carries node attributes:

```
{ skill, inputs: [string], metadata: object, status, result: AgentResult }
status ∈ {"pending","running","complete","failed","skipped"}
```

Public methods:

```
Graph.add_node(skill, inputs, metadata=none) -> nid   // adds edges for each n:<id> input already in graph
Graph.mark(nid, status)
Graph.ready_nodes() -> [nid]                           // pending nodes whose predecessors are all complete|skipped
Graph.has_running() -> bool
Graph.extend_from(src_nid, result, registry) -> [new_nids]
```

##### `extend_from` algorithm (essential — the heart of the graph growth)

```
extend_from(src_nid, result: AgentResult, *, registry):
    added = []
    label_to_id = {}

    # Pass 1 — allocate node ids for every spec in result.successors.
    #          Build a label→id map using spec.metadata.label.
    for spec in result.successors:
        nid = next_id()
        new_node(skill=spec.skill, inputs=<defer>, metadata=spec.metadata,
                 status="pending")
        label_to_id[spec.metadata.label] = nid
        added.append(nid)

    # Pass 2 — now resolve each new node's inputs.
    for spec, nid in zip(result.successors, added):
        resolved_inputs = []
        for inp in spec.inputs:
            # reference to a sibling by label
            if inp == "n:<label>" or (inp is a bare label found in
                                      label_to_id):           → resolve
            # reference to an integer node id already in the graph
            elif inp matches "n:<int>" and exists in graph:    → keep
            # canonical handle
            elif inp == "USER_QUERY" or inp starts with "art:": → keep
            # anything unresolvable → FALL BACK TO src_nid so the child has a
            # structural parent edge (preserves ordering & replay topology)
            elif inp matches "n:<label>" but not yet known:    → src_nid
            else:                                                → keep literal
            resolved_inputs.append(transform)
        add edges for every "n:…" input in resolved_inputs
        set node.inputs = resolved_inputs

    # Fan-out workers with inputs == []  ← DO NOT substitute parent. After the
    # loop, add a parent → child structural edge so topology stays connected.

    # Splice static internal_successors from the SOURCE skill's config
    # (e.g. coder → sandbox_executor). The successor is `add_node(child_skill,
    # inputs=[src_nid])`.

    # Critic auto-insertion
    if registry.get(result.agent_name).critic AND added non-empty:
        for each newly-added child:
            remove the edge src → child
            insert a "critic" node with metadata = {target: src_nid, child}
            add edges  src → critic   and   critic → child

    return added
```

The "fall back to src_nid" rule is critical: it guarantees every child has at
least one upstream edge, which is what makes replay and concurrent dispatch
deterministic.

#### 2.5.2 `Executor.run(...)` — the main loop

Pseudocode (the single most important algorithm in the architecture):

```
async run(query, *, session_id, resume=false, chat_history, chat_context,
          persona, model_profile) -> string:

    set_profile(model_profile)
    sid = session_id or new_run_id("run-")
    store = SessionStore(sid)

    if resume:
        graph = Graph.from_nx(store.read_graph())
        # reset any "running" nodes back to "pending" (we crashed mid-flight)
        for n in graph.nodes: if status == "running": mark pending
        if not query: query = store.read_query()
    else:
        store.write_query(query)
        graph = Graph()
        graph.add_node(skill="planner", inputs=["USER_QUERY"])

    # MEMORY is read ONCE per run and threaded into every skill's prompt.
    memory_hits = memory.read(query, history=chat_history)
    store.write_memory_hits(memory_hits)         # best-effort
    memory.remember(query, source="user_query", run_id=sid)  # best-effort

    formatter_answer = none
    recovered_branches = {}     # target_nid → true (critic-fail cap)
    cap_hit = []                # branches that exhausted the cap

    while true:
        ready = graph.ready_nodes()
        if ready.empty() and not graph.has_running(): break
        if executed_count + len(ready) > MAX_NODES: break

        for nid in ready: Graph.mark(nid, "running")
        store.write_graph(graph)

        outcomes = await gather(_run_one(nid, ...) for nid in ready)

        for (nid, result, prompt) in outcomes:
            # persist NodeState (with prompt_sent = prompt, timestamps…)
            if result.success:
                Graph.mark(nid, "complete")
                store.write_node(NodeState(nid, status="complete", result, prompt))

                if skill == "critic":
                    handled = handle_critic_verdict(nid, result, graph,
                                                     recovered_branches, cap_hit)
                    if handled: continue                      # don't double-extend
                Graph.extend_from(nid, result, registry=registry)

                if skill == "formatter":
                    formatter_answer = result.output["final_answer"]
            else:
                Graph.mark(nid, "failed")
                store.write_node(NodeState(nid, status="failed", result, prompt))
                decision = plan_recovery(failed_skill=skill,
                                          error_text=result.error,
                                          failed_node_id=nid)
                if decision.action == "skip": continue
                # replan ─ adds a new planner node feeding the survivors
                graph.add_node(skill="planner",
                               inputs=[nid],   # or the survivors
                               metadata={ failure_report: decision.failure_report,
                                          recovers: nid,
                                          recovery_reason: decision.reason })

        store.write_graph(graph)

    if formatter_answer is none:
        # best-effort fallback: JSON of the last complete node's output (truncated 2000)
        return json.dumps(last_complete_output)[:2000]
    return formatter_answer
```

`_run_one(nid, …)` ties `Graph.mark(nid,"running")`, `NodeState.started_at`,
`run_skill(...)`, and exception capture together. On exception it returns
`AgentResult(success=false, error=<msg>)` with `prompt="(exception before
prompt-render)"`.

#### 2.5.3 CLI

- `--chat <id>`       selects/creates a specific chat id
- `--persona <text>`  persona token for planner/formatter
- `--model <name>`    one of the configured profiles (validated via
  `agent_model.resolve`); default `gemini`
- `--resume <sid>`    resume a session by id
- no flags + a positional argument ⇒ oneshot (single query, print answer, exit)
- otherwise ⇒ interactive REPL

Interactive commands: `/quit`, `/exit`, `quit`, `exit` to end; `/help`;
`/chat` to show chat id; everything else is dispatched through
`chat.handle_turn`.

**State / Persistence:** heavy — see §6 for the session layout.

**Dependencies:** every other component. `Flow` is the central integrator.

**Porting notes:**
- Use an async runtime (`tokio`). Concurrent node dispatch uses
  `gather`/`join_all`.
- The graph data structure needs only: adjacency lists, a node-id map, a
  per-node attribute dict. Any container library suffices; no special graph
  library is required.
- Resume correctness depends EXACTLY on the `running → pending` reset rule.

---

### 2.6 `mcp_runner` — multi-turn tool-use loop

**Role:** drives the LLM ↔ MCP-tool loop for a single skill invocation that has
tools. Spawn the MCP server as a child process over stdio; loop until the model
stops calling tools or the hop cap is hit.

**Constants:** `MAX_TOOL_HOPS = 6`.

**Public contract:**

```
async run_with_tools(prompt, tools_payload, agent, session_id,
                     provider=none, model=none, provider_pin=none,
                     max_tokens=2048, temperature=0.3) -> reply
```

`reply` has the same shape as a `ChatResponse` (see §3.1): `{ provider, model,
text, tool_calls: [...], stop_reason, … }`.

**Algorithm:**

```
profile = get_chat_kwargs()
use_provider = provider or profile.provider or provider_pin
use_model     = model    or profile.model

messages = [ {role:"user", content:prompt} ]

spawn an MCP stdio child (the mcp_server binary) and ClientSession.initialize()

for hop in 0 ..= MAX_TOOL_HOPS:
    reply = await gateway.chat(messages, tools=tools_payload,
                               tool_choice="auto", agent=agent,
                               session=session_id, provider=use_provider,
                               model=use_model, max_tokens=max_tokens,
                               temperature=temperature)
    last_reply = reply
    if reply.tool_calls empty:
        break             # natural termination
    # append assistant message with tool_calls
    messages.push({ role:"assistant", tool_calls:reply.tool_calls,
                    content:reply.text or null })
    for tc in reply.tool_calls:
        result_text = await mcp_dispatch(tc.name, tc.arguments)
        # CRITICAL CAP: per-tool reply is truncated to 8000 chars
        messages.push({ role:"tool",
                        tool_call_id: tc.id,
                        content: result_text[:8000] })

return last_reply        # if hop cap reached, return what we have
```

**Internal helper `_dispatch_tool(session, name, args) -> string`:** calls
`session.call_tool(name, args)`, concatenate all `content[i].text` items. On
exception returns `json.dumps({"error": <msg>})` (never throws).

**State / Persistence:** none beyond the MCP server's own side effects.

**Dependencies:** gateway (chat), MCP client (stdio transport),
`mcp_server` binary.

**Porting notes:**
- The MCP server can stay in Python (spawn `python mcp_server.py` over stdio).
  The Rust side needs a minimal MCP client that speaks `initialize`,
  `tools/list`, `tools/call` over stdio JSON-RPC.
- Alternatively, re-implement the 11 tools in Rust (see §4) and write a small
  MCP server in Rust. Most tools are plain HTTP/file ops; only
  `fetch_url` (crawl4ai) and `web_search` (Tavily/DDG) need external libs.

---

### 2.7 `mcp_server` — the tool surface

**Role:** an MCP server (JSON-RPC over stdio) exposing 11 tools. Only 3 are
advertised to the model via the skill's `tools_allowed` list (see §5 for how to
change that).

See §4 for the full tool table.

**State / Persistence:** writes Memory facts (via `index_document`),
uses `state/memory.json` and FAISS indirection.

**Dependencies:** the gateway (for embeddings when indexing), the local memory
module, the artifact store.

**Porting notes:** see §4 and §5.

---

### 2.8 `memory` — typed memory store

**Role:** a read/write typed memory service, vector-first with keyword
fallback, with three write surfaces:

1. `remember(raw_text, …)` — LLM classifies the text into one of four kinds.
2. `record_outcome(tool_call, result_text, …)` — deterministic
   `tool_outcome` writer (no LLM).
3. `add_fact(descriptor, value=…, keywords=…, …)` — direct fact writer (used
   by `index_document`).

**Kinds & embedding behaviour:**

| Kind | Embedded? | Notes |
|---|---|---|
| `fact` | yes | the main kind; chunks of indexed docs become `fact`s |
| `preference` | yes | user-stated preferences |
| `tool_outcome` | yes | a tool execution result (compact) |
| `scratchpad` | no | short-lived notes; keyword-only retrieval |

**Read API:**

```
read(query, history=none, *, kinds=none, top_k=8) -> [MemoryItem]
```

Order:
1. Vector search: embed the query (task_type=`retrieval_query`), search FAISS,
   hydrate from the on-disk memory list, filter by `kinds`, cap at `top_k`.
2. If vector returns nothing → keyword fallback over `(descriptor ∪
   keywords)` token sets, optionally boosted with tokens from the last 3 chat
   turns. Score = token-overlap count.

**Write flows:**

```
remember(raw_text, *, source, run_id, goal_id=none):
    ensure_gateway()
    schema = Classification.model_json_schema()        # json_schema response_format
    parsed = gateway.chat(
        raw_text,                                      # exactly the raw text as prompt
        system=<classification system prompt>,
        agent="memory",                                # for cost rollup
        response_format={type:"json_schema",
                         schema, name:"Classification", strict:true},
        temperature=1.0,
        **profile_kwargs)
    if parse fails → fallback Classification(kind=fact,
                                              descriptor=raw_text[:200],
                                              keywords=first10tokens,
                                              value={raw: raw_text})
    if value is empty: value = {raw: raw_text}         # NEVER null
    if kind in EMBEDDABLE: embedding = embed(descriptor, "retrieval_document")
    persist(MemoryItem)

record_outcome(*, tool_call, result_text, artifact_id, run_id, goal_id):
    constructed deterministically — no LLM.  Embed the descriptor.

add_fact(descriptor, *, value, keywords, source, run_id, goal_id=none):
    direct fact write — embed the descriptor.
```

**Persistence rule:** `_persist_item` appends to `memory.json` and, if
embedded, calls `VectorIndex.add(item.id, item.embedding)` then
`VectorIndex.persist()`. Both writes are atomic (temp file + atomic rename).

**State / Persistence:**
- `state/memory.json` — JSON array of `MemoryItem` (indent=2), append-only.
- `state/index.faiss` + `state/index_ids.json` — the parallel vector index.

**Dependencies:** gateway (embed + chat), `vector_index`, `schemas`.

**Porting notes:**
- The classifier prompt returns strict JSON. Validate the result against the
  classification schema. On any failure, fall back deterministically — the impl
  must never error out of `remember()`.
- The "_same `Vec<MemoryItem>` threaded into every node's prompt" rule: caller
  (`Executor.run`) holds the list in the loop variable; do NOT re-read per
  skill. Reading is cheap on the original implementation because Python
  re-loads the file each call — but the architectural intent is one probe per
  run. The Rust port should explicitly load once.

---

### 2.9 `vector_index` — FAISS-like inner-product index

**Role:** a fixed-dim inner-product index over L2-normalised vectors
(=cosine similarity), persisted to two files.

**Public contract:**

```
VectorIndex(store_dir)
.add(item_id : string, embedding : [float])   # L2-normalise; first add fixes dim
.search(query_embedding : [float], k=5) -> [(id, score)]
.persist()
.clear()
.size, .dim
```

**Behaviour:**
- First `add` sets the index dimensionality (`dim`). Subsequent adds with a
  different dimension raise.
- IDs live in a parallel `Vec<string>`. `search` skips negative `indices_returned`
  entries (FAISS returns -1 when fewer than `k` results are available).
- Persistence: write binary index + JSON ids list; atomic via temp files.

**State / Persistence:** `state/index.faiss` (binary), `state/index_ids.json`
(text array). Both reload every time `memory._index()` is invoked (cheap at
small scale; the Rust port can keep it warm).

**Dependencies:** only `numpy`/`faiss` equivalent.

**Porting notes (Rust):** choose one of:
- `faiss` crate (binary-compatible with the on-disk files).
- `hnsw` / `hora` / `instant_distance` (will need a one-time re-index of
  `state/memory.json`).
- `usearch` (good for small dims).
- Embedding model dim is fixed once; do not change after first add.
- **Cosine-by-inner-product:** always L2-normalise before adding or querying;
  that lets `IndexFlatIP`/`dot product` simulate cosine.

---

### 2.10 `artifacts` — content-addressable blob store

**Role:** large outputs (e.g. crawled-page markdown) are spilled to disk and the
agent only carries a short `art:<digest>` handle + descriptor.

**Public contract:**

```
put(blob : bytes, *, content_type, source, descriptor) -> "art:<sha16>"
get_bytes(artifact_id) -> bytes
get_meta(artifact_id) -> Artifact
exists(artifact_id) -> bool
```

**Behaviour:**
- `digest = sha256(blob).hexdigest()[:16]` (32 hex chars truncated — 16 hex).
- `<sha16>.bin` contains raw bytes (deduped: do not rewrite if present).
- `<sha16>.json` contains the `Artifact` metadata.
- Inline-into-prompt cap: 20 KB per artifact in `resolve_inputs`. Larger
  bytes live here.

**State / Persistence:** `state/artifacts/<sha16>.bin` + `.json`.

**Dependencies:** only a hashing primitive.

**Porting notes:** trivially ported to Rust — `sha2::Sha256`, flex hex truncation,
`std::fs::write` with a `.tmp` rename for atomicity.

---

### 2.11 `chat_store` — multi-turn transcripts

**Role:** durable, ordered per-chat conversation. Independent of graph runs:
each user message spawns its own `run-*` graph; the chat stitches runs.

**Chat id rules:** regex `^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$`. Reject `..`,
leading `/` or `\`. Post-resolve root-escape guard.

**Public contract:**

```
ChatTurn   { role: "user"|"assistant", content: string, run_id: string|null,
             ts: datetime }
ChatMeta   { version:1, created_at, updated_at, persona: string|null,
             channel: string|null }

ChatStore(chat_id, *, root=none)
create_or_open(*, persona, channel) -> ChatMeta     # opens+updates or creates
append_turn(turn: ChatTurn)                         # atomic read/append/write
recent_history(*, max_turns=12, max_chars=6000,
               exclude_trailing_user=true) -> [ChatTurn]
history_dicts(...)                                  # [{role, content, run_id}, …]
format_history(...)                                 # "User: …\nAssistant: …"
```

**Behaviour details:**
- `recent_history` drops a trailing unfinished user turn (so the message
  currently being answered isn't duplicated in CHAT HISTORY).
- Trim is from the front to fit the char budget.
- Writes use `_atomic_write` (temp + rename).

**State / Persistence:** `state/chats/<chat_id>/{meta.json, conversation.json}`.

**Dependencies:** the shared `_atomic_write` helper.

**Porting notes:** in Rust implement with `serde_json` and a single `tokio::Mutex`
per chat id (or a router-level mutex). The traversal guard for chat id is
security-relevant (don't skip it).

---

### 2.12 `chat` — per-message adapter on top of the executor

**Role:** the only entrypoint UIs should call. Mounts a run-graph on top of a
chat, runs the executor, appends the assistant response.

**Public contract:**

```
ChatTurnResult { answer: string, chat_id: string, run_id: string }
new_chat_id(prefix="cli") -> string                # prefix + "-" + 8 hex
async handle_turn(chat_id, message, *, persona=none, channel=none,
                  model_profile=none, executor=none) -> ChatTurnResult
```

**Algorithm:**

```
set_profile(model_profile)                          # always first
cid = chat_id or new_chat_id("cli")

# per-chat_id lock so concurrent adapter deliveries cannot interleave writes
lock = lock_for(cid)
async with lock:
    store = ChatStore(cid).create_or_open(persona, channel)
    history_dicts = store.history_dicts(include_trailing_user=true)
    history_text  = store.format_history(include_trailing_user=true)
    persona = arg_persona or meta.persona
    run_id = "run-" + uuid4().hex[:8]
    store.append_turn(user turn with run_id)

    executor = executor or new Executor()           # lazy import to avoid cycles
    answer = await executor.run(text, session_id=run_id,
                                chat_history=history_dicts,
                                chat_context=history_text,
                                persona=persona,
                                model_profile=model_profile)

    store.append_turn(assistant turn with run_id)
return ChatTurnResult(answer, cid, run_id)
```

**State / Persistence:** only via `chat_store` and the executor.

**Dependencies:** `chat_store`, `agent_model`, `flow.Executor` (lazy).

**Porting notes:** locks must be per-`chat_id` to allow concurrent chats
to run safely. Eagerly instantiate `Executor` once and reuse — the lazy
import is a Python-specific workaround.

---

### 2.13 `persistence` — session store + atomic write

**Role:** a per-run directory under `state/sessions/<sid>/`, plus the shared
atomic-write helper.

**Atomic write:**

```
_atomic_write(path, data):
    parent.mkdir(parents=true, exist_ok=true)
    write path.with_suffix(path.suffix + ".tmp")
    rename(tmp, path)             # atomic on POSIX & recent Windows
```

Every file in the project MUST use this helper (chat store, memory,
graph, NodeState). It guarantees a mid-run SIGKILL never corrupts a snapshot.

**SessionStore layout:**

```
state/sessions/<sid>/
├── query.txt               verbatim user query
├── graph.json              networkx/node-link format; edges key="edges"
├── nodes/n_XXX.json        per-node NodeState (for `n:<i>` → n_<i:03d>.json)
└── memory_hits.json        degraded snapshot {id, kind, descriptor, source,
                            chunk, raw} (debugging / viewer)
```

**Public contract:**

```
SessionStore(session_id)
write_query(query); read_query() -> string
write_graph(graph)                # snapshots nodes+edges; AgentResult serialised
read_graph() -> Graph | none      # on JSON missing falls back to legacy pickle
write_node(state: NodeState)
write_memory_hits(hits)
read_node(node_id) -> NodeState | none
read_all_nodes() -> [NodeState]   # sorted glob of n_*.json (completion order)
list_sessions() -> [sid]
```

**Behaviour details:**
- `write_graph` serialises each node's `result` to JSON-friendly form and
  sets `_result_typed=true` so a future reader can reconstruct strong types.
- `read_graph` pops `_result_typed` and re-validates each `AgentResult`.
- `read_all_nodes` skips corrupt files (logs to stderr).

**State / Persistence:** the session directory.

**Dependencies:** shared with `chat_store.py` (Rust: one `_atomic_write`).

**Porting notes:**
- Resume reads `graph.json` and rebuilds the graph; any `running` nodes
  become `pending`; `query.txt` is the fallback for the query.
- The sorted-glob order of `nodes/n_*.json` is the deterministic *completion*
  order; replay uses this.

---

### 2.14 `recovery` — failure policy

**Role:** keep `flow` free of conditionals by centralising the failure
decision table.

**Public contract:**

```
RecoveryReason = "transient" | "validation_error" | "upstream_failure"
RecoveryAction = "skip" | "replan" | "critic_fail"
RecoveryDecision { action, reason, note, failure_report=none }

classify_failure(error_text) -> RecoveryReason
plan_recovery(*, failed_skill, error_text, failed_node_id) -> RecoveryDecision
handle_critic_verdict(nid, result, graph, recovered_branches, cap_hit) -> bool
```

**Decision table:**

| reason            | failed_skill | action |
|-------------------|--------------|--------|
| `transient`       | any          | `skip` (gateway retry already exhausted) |
| `validation_error`| any          | `skip` (prompt bug) |
| `upstream_failure`| `planner`    | `skip` (would loop forever) |
| `upstream_failure`| other        | `replan`, `failure_report = "node=… skill=… reason=… error=…"` |

**Classifier markers** (case-insensitive substring on `error_text`):

- `validation_error`: `malformed`, `validationerror`, `validation error`
- `transient`: `503`,`502`,`504`,`timeout`,`timed out`,`connection`,
  `connectionerror`,`httpstatuserror`,`service unavailable`,`bad gateway`,
  `gateway timeout`
- `upstream_failure`: empty / otherwise

**Critic handler:**

```
handle_critic_verdict(nid, result, graph, recovered_branches, cap_hit):
    if result.output.verdict != "fail": return False       # pass
    target_nid = node.metadata.target  or  first "n:" input
    child_nid  = node.metadata.child   or  first successor
    graph.mark(child_nid, "skipped")

    if target_nid and target_nid not in recovered_branches:
        recovered_branches[target_nid] = true
        graph.add_node(skill="planner",
                       inputs=[nid],
                       metadata={ failure_report:
                                  "critic failed target=… child=… rationale=…",
                                  recovers: target_nid,
                                  recovery_reason:"critic_fail" })
        return True
    else:
        cap_hit.append(target_nid)                          # cap exhausted
        return True                                         # branch stays skipped
```

**State / Persistence:** none.

**Dependencies:** schemas, Graph API.

**Porting notes:** pure function module — trivial to port. Be precise about
the failure_report wording because recovery planners are prompted with it.

---

### 2.15 `sandbox` — subprocess code runner

**Role:** runs the Python code emitted by the `coder` skill in an isolated
child. **Not a security boundary** — only a usability / containment helper.

**Constants:**
```
DEFAULT_TIMEOUT_S = 30
DEFAULT_STDOUT_CAP = 1_000_000      (1 MB)
DEFAULT_STDERR_CAP = 1_000_000
DEFAULT_ENV_WHITELIST = (PATH, HOME, LANG, LC_ALL, LC_CTYPE)
```

**Public contract:**

```
run_python(code, *, timeout_s=30, stdout_cap=1e6, stderr_cap=1e6,
           env_whitelist=DEFAULT_ENV_WHITELIST, extra_env=none) -> object
```

**Returned object:**

```
{
  exit_code: int,
  stdout: string, stdout_truncated: bool,
  stderr: string, stderr_truncated: bool,
  files_written: [{name, size_bytes}],
  timed_out: bool,
  cwd: string                            // informational — tempdir auto-removed
}
```

**Algorithm:**

1. Scrub env: keep only whitelisted vars + `extra_env`.
2. Create a temp directory under the OS temp root, prefix `ollive-sandbox-`.
3. Write `main.py` with `code` (utf-8).
4. Spawn `python main.py` in the temp dir with `stdin` empty + captured
   stdout/stderr, `timeout = timeout_s`.
5. On timeout: `timed_out=true`, `exit_code=-1`, stderr append
   `[sandbox] killed after {Ts}s wall-clock`.
6. Truncate stdout/stderr via head+tail rule:
   if over cap, keep first `cap-200` bytes + "...[truncated; {N} more bytes]…"
7. List the temp dir excluding `main.py`: `files_written=[{name,size_bytes}]`.
8. Return the dict above.

> Note: tempdir is wiped on return; the Rust port should either keep the
> tempdir alive (until the node completes) or `artifacts.put(...)` files the
> coder intends to surface.

**State / Persistence:** transient.

**Dependencies:** OS subprocess API; the `python` interpreter (or whatever
interpreter the Rust-port decides — e.g. keep Python, switch to `cargo run`,
or shell out to Node).

**Porting notes:** a Rust port can keep using a Python subprocess (path
resolved at runtime), or substitute another sandbox runtime — the contract is
"feed code, get stdout/stderr/exit_code/files_written". The
env-whitelist + timeout are usability aids, not a real sandbox. Document this.

---

### 2.16 `replay` — interactive session replay (debugging)

**Role:** stdin-driven viewer for `state/sessions/<sid>/`. Useful for debug
and demo; not required by the agentic loop. Specified for completeness.

**Behaviour:**
- Lists sessions when no arg. Walks `nodes/n_*.json` (glob-sorted = completion
  order).
- For each NodeState prints: index, status, elapsed, provider, retries,
  inputs, error (240 chars), output (500 chars JSON).
- Keys: `enter` advance, `p` show full prompt, `o` show full output, `q` quit.

**Porting notes:** optional. Skip in v1 of the Rust port if you only need the
agentic system.

---

### 2.17 Legacy single-loop modules (optional port)

> The production DAG path does **not** use these. They are described for
> completeness; the Rust port can skip them.

#### 2.17.1 `perception`
LLM-based goal tracker. Returns an `Observation(goals: [Goal])` from a strict
`response_format`. Goals are positional — identified by index, never by id. Has
a lengthy system prompt about decomposition, position-ordering, the
synthesis-keyword rule, the artifact attach rule, etc. Not called by production
code.

#### 2.17.2 `decision`
Decides on each loop iteration whether to answer in text or call a single MCP
tool. Has a long system prompt that explicitly instructs the model to call
`create_file`/`index_document` when needed — which is the mechanism by which
the non-advertised MCP tools could be reached by the legacy path. Not called by
production code.

#### 2.17.3 `action`
"Pure" tool dispatcher. Collapses an MCP `CallToolResult` to a string; if
over `ARTIFACT_THRESHOLD_BYTES = 4096`, spills bytes to the artifact store and
returns a short descriptor + `art:` handle. Blocks decision-hallucinated
`art:` handles being passed as `path`/`url` arguments.

If you do port them, you must also surface them to the gateway by using
`auto_route` ∈ `{perception, memory, decision}` on the chat call (see §3.1).

---

## 3. Gateway HTTP contract (the brain)

The gateway is a separate FastAPI service on `http://localhost:8108`. The agent
talks to it over HTTP **only**. (In Python the original `gateway.py` re-uses
the gateway's client class via `importlib`; the Rust port calls
HTTP directly.)

Env override: `LLM_GATEWAY_URL` / `GATEWAY_PORT`.

### 3.1 `POST /v1/chat` — the central call

#### Request body

| Field | Type | Required | Notes |
|---|---|---|---|
| `messages` | `[{role, content}]` | one of messages/prompt | OpenAI-style; the agent normally sends `prompt` instead |
| `prompt` | string | one of messages/prompt | convenience single-shot |
| `system` | string OR `[{text, cache}]` | no | system prompt; `cache:true` hints the provider to cache it |
| `provider` | string? | no | `gemini`/`nvidia`/`groq`/`cerebras`/`ollama`/`openrouter`/`github` |
| `model` | string? | no | null → provider default |
| `max_tokens` | int | default 2048 | |
| `temperature` | float | default provider-determined | |
| `stream` | bool | default false | SSE if true |
| `tools` | `[{name, description, input_schema}]` | no | only tools listed here are visible to the model |
| `tool_choice` | `"auto"| "none" | {name}` | no | |
| `cache_system` | bool? | no | provider-caching hint for the system block |
| `reasoning` | `"off"|"low"|"medium"|"high"` | no | exclude providers lacking the cap |
| `response_format` | `{type:"json_schema", schema, name, strict}` | no | strict structured output |
| `auto_route` | `"perception"|"memory"|"decision"` | no | runs router-LLM classification first |
| `agent` | string | no | used for cost-by-agent and YAML-based provider pinning |
| `session` | string | no | cost-by-agent session scope (`run-XXXXXXXX`) |

**Provider pinning:** if `agent` is set and `provider` is NOT, the gateway
consults `agent_routing.yaml` for a preferred provider. Caller's explicit
`provider` wins over the pin.

#### Response body

```jsonc
{
  "provider": "gemini",
  "model": "gemini-2.5-flash",
  "text": "...",
  "tool_calls": [{"id":"...","name":"web_search","arguments":{...},"provider_meta":{...}}],
  "stop_reason": "tool_use" | "end_turn" | "max_tokens" | "error",
  "input_tokens": 0, "output_tokens": 0,
  "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0,
  "latency_ms": 0,
  "tool_call_dialect": "native" | "prompted_fallback" | "none",
  "reasoning_applied": false,
  "parsed": { ... },            // only when response_format was provided & validation passed
  "attempted": [{"provider":"groq","reason":"..."}, ...],
  "router_decision": { ... } | null,  // only when auto_route was used
  "retries": 0
}
```

#### Status codes

| Code | Meaning |
|---|---|
| 200 | success |
| 400 | unknown provider |
| 413 | embed input too large |
| 429 | upstream rate-limit when provider pinned |
| 502 | explicit provider failed |
| 503 | (a) `auto_route` HUGE (input > 8000 tokens); (b) all providers down; (c) structured output validation failed; (d) embed 503 with no pin |

#### Server-side retry semantics

- One same-provider retry on transient 5xx/408/timeout, exponential backoff
  capped at 2 s. `retries` is returned in the response.
- One corrective retry for `json_schema`: appends an assistant turn + a
  user "fix it" turn when validation fails first time.

### 3.2 `POST /v1/chat/batch`

Request: `{"calls": [<ChatRequest>, …], "max_concurrency": 4}`
Response: `{"results": [<ChatResponse> | {"error", "status_code"}, …]}` in
input order.

Useful for parallel node dispatch inside a wave — the original implementation
uses one chat call per node, looped via `run_skill`. A batched variant is not
required, but is a clean optimisation for the Rust port.

### 3.3 `POST /v1/embed`

Request: `{"text":"...","task_type":"retrieval_document"|"retrieval_query","provider": null}`
Response: `{"provider":"ollama","model":"...","embedding":[...],"dim":768,
"latency_ms":0,"attempted":[...]}`

The embedding dim is fixed for a given model; the Rust vector index must fix
its dim on first add (see §2.9).

### 3.4 `GET /v1/cost/by_agent?session=<sid>`

Per-agent cost rollup from the SQLite log. `?session=` scopes to one run.

### 3.5 Endpoints the agent NEVER calls (not needed for porting)

`GET /v1/providers`, `GET /v1/capabilities`, `GET /v1/status`, `GET /v1/routers`,
`GET /v1/embedders`, `GET /v1/calls`, `GET /` (dashboard), `GET /help`.

### 3.6 `curl` equivalents of every agent call

```bash
# Plain text-only skill (e.g. distiller)
curl -s -X POST http://localhost:8108/v1/chat \
  -H 'content-type: application/json' \
  -d '{
        "prompt":  "<rendered skill prompt>",
        "system":  "<optional system text>",
        "provider":"gemini",
        "agent":   "distiller",
        "session": "run-abcd1234",
        "max_tokens": 1200,
        "temperature": 0.1
      }'

# Skill with tools (researcher) — multi-turn handled by mcp_runner
curl -s -X POST http://localhost:8108/v1/chat \
  -H 'content-type: application/json' \
  -d '{
        "messages":[{"role":"user","content":"<prompt>"}],
        "tools":[
          {"name":"web_search",
           "description":"Search the web. Hard-capped at 5 results.",
           "input_schema":{"type":"object",
                           "properties":{"query":{"type":"string"},
                                         "max_results":{"type":"integer","default":3}},
                           "required":["query"]}},
          {"name":"fetch_url",
           "description":"Fetch clean markdown from a URL.",
           "input_schema":{"type":"object",
                           "properties":{"url":{"type":"string"}},
                           "required":["url"]}}
        ],
        "tool_choice":"auto",
        "provider":"gemini",
        "agent":"researcher",
        "session":"run-abcd1234",
        "max_tokens":2500,
        "temperature":0.7
      }'

# Structured-output call (memory classify)
curl -s -X POST http://localhost:8108/v1/chat \
  -H 'content-type: application/json' \
  -d '{
        "prompt":   "<raw user query>",
        "system":   "<classification system prompt>",
        "agent":    "memory",
        "temperature": 1.0,
        "response_format": {
          "type":"json_schema",
          "name":"Classification",
          "strict": true,
          "schema": {
            "type":"object",
            "properties":{
              "kind":{"type":"string","enum":["fact","preference","tool_outcome","scratchpad"]},
              "descriptor":{"type":"string"},
              "keywords":{"type":"array","items":{"type":"string"}},
              "value":{"type":"object"}
            },
            "required":["kind","descriptor","keywords","value"]
          }
        }
      }'

# Embedding (memory add/search)
curl -s -X POST http://localhost:8108/v1/embed \
  -H 'content-type: application/json' \
  -d '{"text":"…","task_type":"retrieval_document"}'
```

### 3.7 Gateway-side `agent_routing.yaml`

```yaml
planner:           gemini
researcher:        gemini
distiller:         gemini
summariser:        gemini
critic:            groq
formatter:         gemini
retriever:         github
sandbox_executor:  github
coder:             gemini
browser:           gemini
```

The agent's `agent_model.get_chat_kwargs()` ALWAYS sets `provider=<profile.provider>`,
which overrides the YAML — so in practice the YAML is mostly dormant. **The
Rust port should pick one strategy**: either force the profile (pass
`provider` explicitly, the agent decides routing) OR trust the gateway (pass
only `agent`, let the YAML pick the provider). Both are valid.

---

## 4. MCP server — full tool surface

The MCP server runs as a child subprocess speaking JSON-RPC over stdio
(server name `"ollive-mcp-server"`). It is spawned once per tool-allowed
skill invocation. The lifecycle ends when `run_with_tools` returns.

### 4.1 Transport / protocol details

- **Transport:** stdio. Rust spawns `python mcp_server.py` with
  `stdin`/`stdout` piped.
- **Protocol:** MCP JSON-RPC — `initialize`, `tools/list`, `tools/call`.
- **File descriptor hygiene:** tools that print rich text (e.g.
  `crawl4ai` via `Rich`) may pollute stdout. The Python implementation
  redirects fd 1 → fd 2 during crawl and restores it in a `finally`.
  A Rust port should ensure the same: **nothing must write to stdout
  except MCP JSON-RPC frames.**

### 4.2 Tools exposed by the MCP server

Only 3 tools are advertised to the model via the  `_TOOL_CATALOG` (see §5);
the other 8 exist but are not surfaced from the DAG path. They are useful for
the CLI / the legacy single-loop.

| Tool | Sync/Async | Args | Returns shape | Backend / notes |
|---|---|---|---|---|
| `web_search` (advertised) | sync | `{query, max_results=5}` | `[{title, url, snippet}]` | Tavily primary (`TAVILY_API_KEY`, cap 5); DuckDuckGo (`ddgs.DDGS.text`) fallback. Usage metered in `usage.json` (monthly cap 950 on Tavily). |
| `fetch_url` (advertised) | async | `{url, timeout=20}` | `{status, content_type:"text/markdown", length_bytes, text}` | `crawl4ai.AsyncWebCrawler.arun`. Pulls `raw_markdown`/`fit_markdown`/`cleaned_html`/`html`. |
| `search_knowledge` (advertised) | sync | `{query, k=5}` | `[{id, descriptor, source, chunk, metadata}]` | Calls `memory.read(query, kinds=["fact"], top_k=k)`. |
| `get_time` | sync | `{timezone="UTC"}` | `{iso, human, timezone, offset_hours}` | `zoneinfo.ZoneInfo`. |
| `currency_convert` | sync | `{amount, from_currency, to_currency}` | `{amount, from, to, rate, converted, date, source}` | `https://api.frankfurter.dev/v1/latest`. |
| `read_file` | sync | `{path}` | `{path, size_bytes, content, encoding:"utf-8"}` | sandbox-scoped — path must resolve under `agent/sandbox`. |
| `list_dir` | sync | `{path="."}` | `{path, count, names:[...], entries:[{name,type,size_bytes}]}` | Deliberately **ONE dict** (not a list) so cardinality survives MCP truncation. |
| `create_file` | sync | `{path, content}` | `{ok:true, path, size_bytes}` | Errors if file exists or parent missing. |
| `update_file` | sync | `{path, content}` | `{ok:true, path, size_bytes}` | Errors if file missing. |
| `edit_file` | sync | `{path, find, replace, replace_all=false}` | `{ok:true, path, replacements, size_bytes}` | Errors if `find` not found or count > 1 without `replace_all`. |
| `index_document` | sync | `{path, chunk_size=400, overlap=80}` | `{path, source, chunks_indexed, chunk_size, overlap}` | Reads `path` or `art:<id>`, sliding-window chunks by word count (`stride=chunk_size-overlap`), `memory.add_fact(...)` per chunk → each chunk gets a FAISS embedding via `gateway.embed`. |

**Chunker** (`_chunk_text(text, size=400, overlap=80)`):
`words = text.split(); stride = max(1, size-overlap); append
" ".join(words[i:i+size])`.

**Path-safety** (`_safe(path) -> Path`): resolves under the sandbox dir,
raises if the resolved path escapes.

**Usage metering:** monthly-rollover counter in `usage.json` with shape
`{month, tavily:{count,errors}, duckduckgo:{count,errors}}`,
`MONTHLY_CAP = 950` on Tavily, guarded by a threading lock.

### 4.3 Per-tool reply cap

In `mcp_runner`, every tool result piped into the next chat call is
truncated to **8000 chars** (`result_text[:8000]`). This prevents one giant
`fetch_url` markdown from breaking the next model call. Implement this cap in
the Rust port.

### 4.4 Side effects requiring shared filesystem

The MCP subprocess must share the filesystem with the agent process
because `index_document` writes Memory facts (→ `state/memory.json`,
`state/index.faiss`, `state/index_ids.json`). If you split the two into
separate machines (e.g. Rust agent + remote MCP server), you must remove
`index_document` from the MCP server and call the memory service over HTTP.

---

## 5. How to add / swap tools in the new architecture

This is the central extensibility point you wanted guidance on. The Rust
architecture should make this the *single* place a contributor touches when
adding a new tool.

### 5.1 The 2-layer distinction (preserve this in Rust)

A tool has TWO independent definitions:

1. **Implementation** (in the MCP server, §4) — the actual executable
   function, its name, arguments schema, return shape.
2. **Visibility** (in `_TOOL_CATALOG` inside `skills`) — the OpenAI-shaped
   descriptor the model is allowed to call.

A tool with no implementation can't run. A tool with implementation but no
catalog entry **exists but the model can never invoke it** (this is how all 8
non-advertised tools work today).

### 5.3 Step-by-step: adding a new tool

1. **Implement** the tool in the MCP server (a new method registered with the
   MCP server's tool registry). See §5.5 for the Rust-MCP-server approach.
   - Decide sync vs async.
   - Declare argument schema as a JSON-schema object (type, properties,
     required).
   - Return shape MUST be JSON-serialisable. If a call returns a list of
     many items, wrap it in a single dict (`count`, `items`, etc.) so the
     cardinality survives MCP truncation — see `list_dir` as the canonical
     example.
   - If the output may be huge, offload it to the artifact store and return
     a descriptor + `art:` handle (this is what `fetch_url` should do for
     very long pages, see `action._result_to_text` for the threshold rule:
     `ARTIFACT_THRESHOLD_BYTES = 4096`).
   - Reuse sandbox path-safety: never accept an arbitrary path; always
     resolve-and-verify it stays under the sandbox dir.

2. **Name** the tool with a lowercase snake_case identifier (e.g.
   `web_search`, `currency_convert`, `read_file`). The model uses this name
   verbatim in `tool_calls`.

3. **Write the visibility entry** in the skill's `tool_payload` catalogue:

   ```json
   {
     "name": "<tool>",
     "description": "<one sentence; the model relies on this>",
     "input_schema": { "type":"object", "properties": { ... }, "required": [...] }
   }
   ```

   This is the descriptor advertised to the model. Keep descriptions short
   and specific ("Hard-capped at 5 results" beats "Search the web").

4. **Attach to a skill** by listing the tool name in that skill's
   `tools_allowed` in `agent_config.yaml`. A model instance of that skill
   will now see the tool.

5. **Update prompts** for the skill that uses the new tool — the prompt
   should mention the tool by name, when to call it, and the expected
   output shape.

6. **Update the JSON output contract** for the skill — the skill's prompt
   still dictates the final structured output the model returns (e.g. the
   researcher emits `{question, sources, findings}`). Tools do NOT change
   the output schema; they only fetch input data.

### 5.4 Step-by-step: removing / replacing a tool

- Remove the catalogue entry (step 3) — the model won't see it any more. The
  implementation can stay in the MCP server harmlessly.
- To fully remove the tool: delete the implementation AND every skill's
  `tools_allowed` entry AND the catalogue entry.
- Keep the `tool_payload(name)` filter simple: an unknown `name` is silently
  dropped (do NOT raise — keeps skill configs forward-compatible).

### 5.5 Approach for the Rust port — three viable options

Pick based on effort vs. ceremony:

| Option | How to add a tool | Pros | Cons |
|---|---|---|---|
| **A. Keep Python MCP server** | new Python function in `mcp_server.py`; Rust spawns it over stdio | zero porting surface; full parity on day one | keeps a Python dependency for the agent |
| **B. Rust MCP server** | new Rust `async fn` registered with the `rmcp`/`mcp-rust` server; same catalogue wiring | all-Rust agent; no Python at all | you must reimplement web_search (Tavily+DDG) & fetch_url (crawler); other 9 are trivial |
| **C. Direct call (no MCP at all)** | tool becomes a Rust `async fn` invoked synchronously inside the agent; no JSON-RPC | simplest; no transport | loses the clean agent/tool process boundary (any tool crash kills the agent) |

> **Recommendation for the Rust port:** Option **A** for v1 (keep Python MCP
> server as-is), then move to Option **B** once v1 works. The MCP server
> process boundary is exactly the seam that lets you do this gradually.

### 5.6 Tool-input trust rules (preserve in any port)

- Never let a model pass an `art:<sha>` handle as a `path` / `url` argument.
  A tool that looks up content handles artifact IDs explicitly.
- Sandbox-scoped tools (`read_file`, `list_dir`, `create_file`,
  `update_file`, `edit_file`, `index_document`) MUST refuse any path that
  escapes the sandbox root.
- Period metering: heavy/external tools (`web_search`, `fetch_url`) should
  count against a monthly cap per provider. The Rust port should keep
  `usage.json` or a `.sqlite` counter.

### 5.7 Tool catalogue (the only visibility source) — keep it separate

In the Rust port, the catalogue should live in a single source file or YAML
(e.g. `tools.yaml`) rather than buried inside the skills module. Concretely:

```yaml
# tools.yaml — Rust port suggestion
- name: web_search
  description: "Search the web (Tavily primary, DDG fallback). Hard-capped at 5 results."
  input_schema:
    type: object
    properties: { query: {type: string}, max_results: {type: integer, default: 3} }
    required: [query]
  implementation: mcp::web_search     # location of the impl
  advertised: true
  meter: tavily
- name: fetch_url
  description: "Fetch clean markdown from a URL via crawl4ai."
  input_schema:
    type: object
    properties: { url: {type: string} }
    required: [url]
  implementation: mcp::fetch_url
  advertised: true
- name: search_knowledge
  description: "Vector search over the agent's indexed knowledge base."
  input_schema:
    type: object
    properties: { query: {type: string}, k: {type: integer, default: 5} }
    required: [query]
  implementation: mcp::search_knowledge
  advertised: true
```

Skills reference tools by name only:

```yaml
# agent_config.yaml (Rust)
researcher:
  prompt: prompts/researcher.md
  tools_allowed: [web_search, fetch_url]
  max_tokens: 2500
  temperature: 0.7
```

The skill runner then asks the catalogue for the visibility descriptors and
sends them as the `tools` array on every chat call. This decoupling is the
main architectural improvement you can adopt in the Rust port without
changing semantics — it makes the tool surface a first-class data file.

---

## 6. Filesystem state layout (essential to replicate exactly)

```
state/
├── sessions/<run-id>/                one directory per flow-run
│   ├── graph.json                    🗲 node-link JSON (see §2.13)
│   ├── query.txt                     verbatim user query
│   ├── memory_hits.json              degraded snapshot, debugging only
│   └── nodes/n_001.json …            per-node NodeState (§2.13)
├── chats/<chat-id>/
│   ├── meta.json                     {version:1, created_at, updated_at, persona, channel}
│   └── conversation.json             ordered list[ChatTurn]
├── artifacts/
│   ├── <sha16>.bin                   raw bytes (content-addressable)
│   └── <sha16>.json                   Artifact meta
├── memory.json                       append-only list[MemoryItem] (indent=2)
├── index.faiss                       binary inner-product index (FAISS)
└── index_ids.json                    parallel list[str] of "mem:XXXXXXXX"
```

### 6.1 Atomic write rule (universal)

Every file in this tree is written via:
```
parent.mkdir(parents=true, exist_ok=true)
write tmp file at path.with_suffix(path.suffix + ".tmp")
rename(tmp, path)                     # atomic on POSIX
```
A mid-write SIGKILL never corrupts a prior snapshot. Replicate this rule
absolutely — it is what makes "resume" viable.

### 6.2 Resume rules

`Executor.run(resume=true, session_id=<sid>)`:
1. Load `graph.json`, rebuild the graph, set every `running` node back to
   `pending`.
2. Use `query.txt` if no `query` is passed.
3. Skip the `planner` seed (graph already has the planner node).
4. Memory is re-read fresh in the resumed run (the per-run probe is re-done).

### 6.3 Replay rules

`replay <sid>` walks `nodes/n_*.json` in sorted-glob order =
completion order. The order is deterministic because `write_node` is invoked
immediately after `mark(complete|failed)`.

---

## 7. Prompt-template contract

Each skill has one `prompts/<skill>.md` file. The file is loaded verbatim and
then `render_prompt` appends blocks:

| Block | When added | Content |
|---|---|---|
| skill template | always | verbatim text of `prompts/<skill>.md` (rstripped) |
| PERSONA | if `persona` set AND skill ∈ `{planner, formatter}` | `PERSONA:\n<persona>` |
| CHAT HISTORY | if `chat_context` set AND skill ∈ `{planner, formatter}` | `CHAT HISTORY (previous turns):\n<text>` |
| USER_QUERY | always a `"USER_QUERY"` token appears in `resolved` | `USER_QUERY: <query>` |
| QUESTION | if `metadata.question` set | `QUESTION: <question>` |
| FAILURE | if `failure_report` set | `FAILURE:\n<failure_report>` |
| MEMORY HITS | if `memory_hits` non-empty | `MEMORY HITS (<n> from FAISS):\n<formatted hits>` |
| INPUTS | always | `INPUTS:\n<json.dumps(resolved, indent=2) cap 20_000>` |

The model replies with **strict JSON** matching each skill's contract:

| Skill | Expected JSON |
|---|---|
| planner | `{"rationale": "...", "nodes": [{"skill": ..., "inputs": [...], "metadata": {"label": ..., "question": ...}}]}` |
| retriever | `{"found": bool, "chunks": [{"source","preview"}], "summary": "..."}` |
| researcher | `{"question": "...", "sources": [{"url","title"}], "findings": "2–6 paragraphs"}` (use `"(not found)"` if unanswered) |
| distiller | `{"fields": {"<name>": "<value>"}, "rationale": "..."}` |
| summariser | `{"summary": "...", "preserved_facts": ["..."]}` |
| critic | `{"verdict": "pass"\|"fail", "rationale": "..."}` |
| formatter | `{"final_answer": "..."}` |
| coder | `{"code": "<python>", "rationale": "..."}` |
| sandbox_executor | (LLM path rarely taken) `{"summary": "..."}` |

Rules embedded in the planner prompt (must be preserved when porting):

- Reference upstream nodes as `n:<label>` (the label is supplied in
  `metadata.label`).
- The final node must be a `formatter`.
- For fan-out workers (which don't depend on a single upstream result), do
  NOT list `USER_QUERY` in `inputs`; instead scope the question via
  `metadata.question`.
- Emit one node per concrete item when comparing/fetching N items.
- Insert a `critic` between a writer and the formatter when strict format
  constraints are present (the orchestrator also auto-inserts critics where
  the config flags `critic: true`).

### 7.1 How to modify a skill in the Rust port

- Edit `prompts/<skill>.md` for the prompt change.
- Edit `agent_config.yaml` for the catalog fields (`tools_allowed`,
  `temperature`, `max_tokens`, `internal_successors`, `critic`).
- Add/swap tools as per §5.
- The orchestrator never changes when you change a skill's behaviour, because
  the orchestrator only knows skill names and is tool-blind.

---

## 8. Rust porting recipe

A concrete crate / module plan:

### 8.1 Workspace layout

```
ollive-rs/
├── Cargo.toml                 workspace
├── crates/
│   ├── ollive-core/           schemas, agent_model, persistent state structs
│   ├── ollive-gateway/        HTTP client (reqwest) wrapping /v1/chat, /v1/embed
│   ├── ollive-memory/         memory + vector_index (faiss or hnsw)
│   ├── ollive-artifacts/      content-addressable store
│   ├── ollive-chat/           chat_store + handle_turn
│   ├── ollive-mcp-client/     stdio MCP client (initialize/tools/list/tools/call)
│   ├── ollive-mcp-server/     reimplementation of mcp_server.py (Rust, optional)
│   ├── ollive-sandbox/        subprocess runner (Python, Node, or whatever)
│   ├── ollive-skills/         skill registry, prompt rendering, tools catalogue
│   ├── ollive-flow/           the DAG executor
│   └── ollive-cli/            main binary
└── config/
    ├── agent_config.yaml
    ├── tools.yaml             (extracted from skills; see §5.7)
    └── prompts/*.md
```

### 8.2 Crate choices

| Concern | Recommended crate | Notes |
|---|---|---|
| Async runtime | `tokio` | required for `gather` semantics |
| HTTP client | `reqwest` w/ `json` feature | keep-alive client per process |
| JSON | `serde_json` + `serde` derive | exact field names match §2.1 |
| YAML config | `serde_yaml` or `yaml-rust2` | |
| Schemas / validation | `schemars` for the response_format JSON schema | |
| FAISS | `faiss` crate | OR `usearch` / `hnsw` / `instant_distance` |
| MCP (client) | `rmcp` or hand-rolled stdio JSON-RPC | minimal: `initialize`, `tools/list`, `tools/call` |
| MCP (server) — optional | `rmcp` server over stdio | only if you choose Option B in §5.5 |
| Storage | `std::fs` + `tempfile` for atomic writes | `tempfile::NamedTempFile::persist()` |
| UUIDs | `uuid` v4 (8 hex truncation) | |
| Hashing | `sha2::Sha256` | truncate to first 16 hex chars |
| Datetime | `chrono` w/ UTC | ISO-8601 strings in JSON |
| CLI parsing | `clap` derive | |
| Web search — `web_search` | `reqwest` to Tavily + `ddgs` API | you may write a tiny ddg client; no Rust crate is canonical |
| Web crawl — `fetch_url` | `reqwest` + `scraper` + naïve html→md | parity with crawl4ai is hard; using `readability`+`html2text` is acceptable for v1 |

### 8.3 Build / run steps (sketch)

```bash
# terminal 1 — start the Python gateway (unchanged)
cd gateway && uv run main.py

# terminal 2 — start the Rust agent
cd ollive-rs
cargo run -- --chat cli-abc --model gemini
# or one-shot:
cargo run -- "<query text>"
# or resume:
cargo run -- --resume run-abc12345
# index the same sandbox corpus:
cargo run --bin ollive-index -- sandbox/kb
```

### 8.4 Decisions to make explicitly (and document)

1. **Provider pinning model** — force the profile (pass `provider` always) OR
   trust the gateway YAML (pass `agent` always). Pick one and stick to it.
2. **MCP surface** — Option A / B / C from §5.5.
3. **Recurrence of `python` for sandbox_exec** — keep Python, or substitute
   another interpreter for `coder`. Document the choice.
4. **The DAG data structure** — hand-rolled adjacency list, `petgraph`
   (Rust crate), or `egraph`-style. Any container works.
5. **Graph serialisation format** — you can replace `networkx node-link` with
   your own JSON; the only requirement is that resume reads back a graph
   carrying `AgentResult.output` per node.

### 8.5 Minimal viable Rust port (first sprint)

The smallest end-to-end loop that exercises the architecture:

1. `ollive-gateway`: `/v1/chat` text-only (no tools).
2. `ollive-skills`: load `agent_config.yaml` + `prompts/planner.md` +
   `prompts/formatter.md` only.
3. `ollive-flow`: `Graph` + `extend_from` + `Executor.run` to completion.
4. `ollive-persistence`: `SessionStore` with `graph.json`+`query.txt`+nodes.
5. `ollive-cli`: REPL that calls a minimal `handle_turn`.

Once that loop produces a `final_answer`, add: memory, artifacts, MCP client,
retriever/researcher/distiller/critic/coder/sandbox skills, recovery.

---

## 9. Validity checklist (use this to confirm portability)

A Rust port passes this checklist iff:

- [ ] A one-shot query with only `planner` + `formatter` skills produces a
      `final_answer` string returned by `handle_turn`.
- [ ] A research-style query dispatches at least one `researcher` node that
      makes a real `web_search` call and the planner can extend from its output.
- [ ] State directory exists:
      `state/sessions/<run-id>/{graph.json, query.txt, nodes/n_001.json}`.
- [ ] Killing the agent mid-run then starting `--resume <sid>` continues the
      graph (the `running → pending` reset is observable).
- [ ] A `critic`-failed branch is skipped; the recovered branch gets exactly
      one recovery planner; a second critic fail is capped.
- [ ] `memory.read` returns hits that include `fact`s indexed via
      `index_document`.
- [ ] `vector_index` dims are fixed; switching the embedding model
      (different `dim`) is rejected.
- [ ] All file writes are atomic (temp + rename); no `.tmp` files leak.
- [ ] `MAX_NODES = 60` is enforced; large graphs terminate and return the
      last-complete JSON instead of hanging.
- [ ] Adding a new tool requires editing only `tools.yaml` (catalogue) +
      `mcp_server` impl (or `ollive-mcp-tools` Rust module) +
      `agent_config.yaml` (`tools_allowed`) + the skill's prompt — and
      never touches `flow` or `skills` source code.

---

## 10. Out of scope (explicitly excluded)

The following parts of the original repository are **not** part of this
specification and do **not** need to be ported:

- The gateway implementation itself (Rust talks to it over HTTP).
- The evaluation suite under `evals/`.
- The interactive dashboards under `gateway/static/` and `graph-viewer.html`.
- The legacy perceive/decide/act loop in `perception.py`, `decision.py`,
  `action.py` — unless the new architecture explicitly wants to add that loop
  as an alternative cognitive layer.
- `replay.py` itself is optional for v1 of the Rust port.
- `browser.md` skill (stub in the original).