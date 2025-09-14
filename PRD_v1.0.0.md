# FINAL PRD v1.0.0 — **reviewlens**

*Intelligent Code Review, Debugging & Security Agent (CLI-first, privacy-by-default)*

**Version:** 1.0.0 (RC-ready)
**Owner:** Head of Product & Principal Engineer (you)
**Date:** September 14, 2025
**License:** Apache-2.0

---

## 1) Vision & Product Promise

**Vision.** Make code reviews **10× faster** and **safer** by combining precise, rules-first static analysis with context-aware AI—without vendor lock-in or privacy compromises.

**Developer Promise.** Install in minutes, run locally or in CI, get a crisp Markdown report + optional PR Checks/Suggestions with **minimal diffs you can accept**. Default private. No source code leaves your machine unless explicitly enabled.

---

## 2) North Star & Guardrails

* **North Star (Activation):** ≥ **40%** of **first-time repos** produce ≥ **1 accepted suggestion** within **24h** of installation.
* **Guardrails:**

  * **p95** end-to-end review ≤ **60s** for typical PRs (≤300 changed LOC, ≤50 files).
  * **False-positive rate** (per finding on fixtures) ≤ **15%**.
  * **Helpfulness** micro-NPS ≥ **4/5** from pilot reviewers.
  * **Privacy:** default **offline** (no external LLM calls). **Zero** secret-leak false negatives on golden set.

---

## 3) Target Users

* **Primary:** Staff/Principal engineers, AppSec-minded reviewers, repo maintainers.
* **Secondary:** EM/Tech Leads aiming to increase throughput while reducing security escapes.

---

## 4) Developer Experience (DX) Goals

* **Time-to-first-value:** ≤ **5 minutes** from install to first report.
* **One file to trust:** `review_report.md` (plus concise stdout summary & CI exit codes).
* **Zero surprise:** explicit config precedence (**flags > env > file**), deterministic CI mode.
* **Safety by default:** provider=`null`, redaction on, diff-only scanning, easy opt-out paths.

---

## 5) Scope v1.0.0

### 5.1 Must-Have (ship in RC)

* **CLI commands:** `check`, `index`, `print-config`, `version`.
* **Rules (high-precision):**

  1. **Secrets/Credentials** (generic + AWS-like patterns)
  2. **SQL Injection (Go)** — unsafe concatenation in `db.Query*/Exec`
  3. **HTTP Client Timeouts/Context (Go)** — missing `Timeout` or `context` on outbound I/O
* **Context (RAG-lite):** local conventions sampler (error handling/logging/signature patterns) — no external store; JSON.zst on disk.
* **Reports:** Markdown + optional JSON; hotspots ranking; **minimal diffs** with rationale; run metadata.
* **CI & PR:** GitHub Action template; Checks summary; suggested patch comments (inline) when enabled.
* **LLM adapters:** `null` (default), OpenAI (flag-gated). Model pinned + `temperature=0` in CI.
* **Privacy:** redaction before prompts; no telemetry unless opt-in.
* **Packaging:** macOS (arm64/x86\_64) + Linux (arm64/x86\_64) binaries; **SHA256** + **cosign** signatures; **Homebrew tap**; **Docker image** for CI.

### 5.2 Should-Have

* Per-rule toggles, severity thresholds, path allow/deny globs.
* Per-finding suppression comment: `// reviewlens:ignore <rule> [reason]`.
* Caching of parsed files and index across runs.
* Progress bars locally; quiet non-TTY in CI.

### 5.3 Won’t (v1)

* Deep DAST, SCA/SBOM, IDE plugins, Windows builds, GitLab bot. (Can run in GitLab CI using Docker.)

---

## 6) User Stories & Acceptance Criteria

### A. Install & First Run

* **As a developer**, I can install and run a review in under 5 minutes.
  **AC:**

  * `brew install reviewlens/tap/reviewlens` **or** `curl -fsSL ... | sh` installs a signed binary.
  * `reviewlens check --diff auto` produces `review_report.md` and exit code **0/1**.
  * `reviewlens print-config` shows effective config + compiled providers.

### B. CI Integration

* **As a maintainer**, I can add a single job to my CI to gate merges.
  **AC:**

  * GitHub Action example works copy-paste; artifact upload enabled.
  * Exit codes: `0` (pass), `1` (findings ≥ fail level), `2` (config), `3` (runtime).
  * Checks summary appears with counts; inline suggestions appear when enabled.

### C. Signal Over Noise

* **As a reviewer**, I only see high-value, actionable findings.
  **AC:**

  * Rules report file\:line, severity, rationale, **minimal diff**, and references.
  * Default **fail level = high**; `--fail-on` overrides.
  * Fixtures show **precision ≥85%**, FP ≤15%.

### D. Privacy & Determinism

* **As a security-conscious team**, I can prove no code left my env by default.
  **AC:**

  * provider=`null`; prompts redacted if enabled; model+driver IDs & versions logged in report.
  * CI mode pins model; `temperature=0`; budgets enforced.

---

## 7) Product Experience Details

### 7.1 Quickstart

```bash
# macOS
brew tap reviewlens/tap
brew install reviewlens

# Linux/macOS (curl)
curl -fsSL https://raw.githubusercontent.com/reviewlens/reviewlens/main/install.sh | sh

# First run
reviewlens check --diff auto
# -> review_report.md, stdout summary, exit code
```

**GitHub Action (copy-paste):**

```yaml
name: reviewlens
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: reviewlens/action@v1
        with:
          args: "check --diff auto --fail-on high --format md"
      - uses: actions/upload-artifact@v4
        with: { name: review_report, path: review_report.md }
```

### 7.2 CLI Surface

```
reviewlens check [--path .] [--diff auto|<ref>] [--base-ref <ref>]
                 [--fail-on critical|high|medium|low]
                 [--format md|json] [--only-changed] [--ci] [--no-progress]

reviewlens index [--path .]
reviewlens print-config
reviewlens version
```

### 7.3 Config (precedence: flags > env > file)

`reviewlens.toml`:

```toml
[llm]
provider = "null"        # null|openai
model = "gpt-4o-mini"    # used only when provider != null

[generation]
temperature = 0.0
max_tokens = 200_000

[privacy]
redaction = true

[paths]
allow = ["**/*.go", "**/*.tf", "**/*.yml", "cmd/**", "internal/**"]
deny  = ["**/vendor/**", "**/node_modules/**", "**/.git/**"]

[rules]
secrets = { enabled = true, severity = "high" }
sql_injection_go = { enabled = true, severity = "critical" }
http_timeouts_go = { enabled = true, severity = "medium" }

[ci]
fail_on = "high"
```

### 7.4 Output Contract

* **Stdout (CI-friendly):**

```
reviewlens: 3 findings (1 critical, 2 high)  p95=41s  fail-on=high → EXIT 1
Report: review_report.md
```

* **`review_report.md`** sections: Summary → Security Findings (with minimal diffs) → Conventions Deviations → Hotspots (top-5) → Appendix (run metadata: ruleset, versions, timings, budgets).

---

## 8) Engineering Requirements

### 8.1 Non-Functional (SLOs)

* **Performance:** p50 ≤ **25s**, p95 ≤ **60s** (typical PR).
* **Resources:** p95 **RAM ≤ 1.5 GB**; CPU multi-core by default.
* **Determinism:** CI pins model IDs; `temperature=0`; time budgets enforced.
* **Portability:** macOS & Linux (arm64/x86\_64).
* **Reliability:** graceful partial results on time budget exceed; non-zero exits mapped clearly (2/3).

### 8.2 Security/Privacy

* Default offline; explicit opt-in for LLM calls.
* Redaction via deterministic pattern matching; redact **only** prompts & public report—**never** rule inputs.
* Min-scope tokens; signed artifacts; `SECURITY.md` with disclosure channel.

### 8.3 Observability

* Local logs with `--verbose`; structured JSON logs in `--ci`.
* (Opt-in) minimal telemetry events: `run_started`, `run_finished`, counts & timings (no source).
* `print-config` exposes effective config & compiled providers.

---

## 9) Metrics & Evaluation

* **Fixture Harness (`make eval`)** collects: precision/FPs per rule, p50/p95 runtime, memory P95.
* **KPI Dashboard:** Activation (D1), Accepted Suggestions %, p95 latency, error rate.
* **A/B (flag-gated):** onboarding copy, default `fail_on`, rule thresholds—guarded by SRM checks.

---

## 10) Release Artifacts & Distribution

* GitHub Releases: macOS/Linux binaries for arm64/x86\_64 + `.sha256` and **cosign** signatures.
* **Homebrew tap** formula auto-updated from release.
* **Docker image** (`ghcr.io/reviewlens/cli:v1`) for CI.
* Docs hosted with copy-paste **Quickstart**, **Rules Reference**, **Config**, **CI Setup**, **Troubleshooting**.
* **Changelog** per release; rollback plan documented.

---

## 11) Release Gates (must be true to tag **1.0.0-RC**)

**Functionality**

* CLI commands implemented (`check`, `index`, `print-config`, `version`).
* Rules: Secrets, SQLi-Go, HTTP-timeouts-Go with per-rule tests & references.
* Report includes minimal diffs & hotspots; JSON export optional.
* GitHub Action example + Checks summary + (flag-gated) inline suggestions.

**Quality & Perf**

* Fixtures: **precision ≥85%**, FP ≤15%; p50 ≤25s, p95 ≤60s; RAM p95 ≤1.5 GB.

**Determinism/Privacy**

* Default provider=`null`; redaction on; CI pins model & temperature=0; run metadata recorded.

**DX & Distribution**

* Signed artifacts + checksums; Homebrew formula; Docker image present.
* `--help`/`print-config` snapshot tests pass; docs complete; install works on clean hosts.

**Pilot Evidence**

* Ran in **≥3** pilot repos for **≥2 weeks**; **helpfulness ≥4/5**; **≥2** critical issues blocked pre-merge.

---

## 12) Risks & Mitigations

* **Noise erodes trust** → restrict to 3 high-precision rules; conservative defaults; easy suppressions.
* **Latency on big diffs** → diff-only, parallel scanning, caching, hard budgets.
* **Privacy concerns** → offline default, explicit toggles, visible run metadata.
* **Vendor drift/lock-in** → adapter layer; null provider first; pinned models in CI.
* **Packaging friction** → `cargo-dist`, signed artifacts, Homebrew tap, Docker CI image.

---

## 13) Appendices (Specs)

### 13.1 Exit Codes

* `0` pass; `1` findings ≥ fail level; `2` config/usage; `3` runtime failure.

### 13.2 Suggested Patch Format (inline)

* GitHub suggestion blocks in MD; single-hunk minimal diffs; guarded by `--allow-suggest`.

### 13.3 Events (opt-in telemetry schema)

* `run_started { commit, files_changed, loc_changed }`
* `finding { rule, severity }` (counts only)
* `run_finished { p50, p95, mem_p95, findings_total }`

---

### Final Note

This v1.0.0 focuses on **precision, speed, and trust**. It is intentionally CLI-first with a thin, delightful PR experience. Everything that risks noise or privacy is off by default and added only when it’s measurably safe and useful.