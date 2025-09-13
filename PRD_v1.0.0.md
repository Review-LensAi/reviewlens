# Product Requirements Document (PRD)

**Product**: Intelligent Code Review Agent (CLI-first)
**Version**: **1.0.0**
**Status**: **Final — Ready for 1.0.0-RC cut**
**Date**: September 13, 2025
**Owner**: Benediktus (PM) — with Lead Architect support

---

## 1) Vision & Problem Statement

### Vision

A context-aware, security-first **AI code review agent** that runs locally or in CI to summarize changes, find high-impact issues (not noise), and propose minimal, reviewable fixes — **without vendor lock-in** to any single LLM.

### Problem

Code review has become a bottleneck. Large, AI-generated PRs overwhelm human reviewers; subtle architectural and security flaws slip through. Traditional scanners are noisy and context-blind. Teams need a reviewer that understands **their** codebase, flags **what matters**, and respects **privacy**.

### Core Hypothesis (MVP)

> With deep repository context (RAG-lite) + high-signal security rules + targeted LLM assistance, we can reduce review time by 5–10× while **improving** security/quality and **lowering** false positives.

---

## 2) Goals & Non-Goals

### Goals (MVP)

1. **Accelerate reviews**: actionable summary + hotspots in < 5 minutes per PR.
2. **Catch critical issues early**: OWASP-focused SAST-lite + secret/credential leak detection.
3. **Context-aware suggestions**: align with repo conventions using a lightweight index.
4. **Actionable output**: Markdown report + minimal diffs, clear rationale.
5. **Provider-agnostic**: support GPT, Claude, DeepSeek, and local/self-hosted models via a pluggable driver.
6. **Privacy-first**: default to offline/null-LLM mode; optional redacted calls to external LLMs.

### Non-Goals (MVP)

* Inline PR comments/Checks API bot (post-MVP).
* DAST, full SCA, or deep language coverage (start with **Go** + generic patterns).
* Autonomous code commits (guard-railed remediation comes later).
* IDE plugins.

---

## 3) Success Metrics (with methodology)

**Primary**

* **P50 end-to-end runtime**: ≤ **5 minutes** per **typical PR** (≤ \~300 changed LOC, ≤ 50 files) on baseline machine.
* **False positive rate**: ≤ **15%** (per-finding) on curated fixtures.
* **Reviewer usefulness**: ≥ **4/5** average rating in pilot teams.
* **Security value**: ≥ **5** critical issues blocked pre-merge across pilots within 60 days of GA.
* **Adoption**: ≥ **10 active repos** run the CLI in CI within **30 days post-GA**.

**Methodology**

* **Precision** = TP / (TP+FP) on a **per-finding** basis using labeled fixtures.
* **FP rate** = FP / (TP+FP).
* **Runtime**: measured on **macOS arm64 (8 cores)** with warm and cold index; report **P50/P95**.

---

## 4) Personas

* **Priya — Senior Reviewer**: cares about architecture, wants high-signal summaries.
* **Leo — Junior Author**: wants concrete, minimal diffs and rationale.
* **Asha — Eng Manager**: wants faster cycle time and fewer regressions.
* **Rizal — AppSec Lead** (secondary): wants fewer secret leaks and earlier detection.

---

## 5) Scope (MVP)

### In Scope

* **CLI**: `reviewer check --path . --diff <ref>` (local & CI).

  * `--diff` defaults to upstream base (auto-detected); override with `--base-ref <ref>`.
* **Outputs**: `review_report.md` + stdout summary; CI-friendly exit codes.
* **Security checks (SAST-lite, Go-focused + generic)**

  * **Secrets/Credentials** (keys, tokens, AWS-like/Generic patterns).
  * **SQL Injection (Go)**: unsafe string concatenation in DB queries.
  * **Unsafe Networking/Timeouts (Go)**: outbound I/O lacking context/cancellation or timeouts.
  * **Server-side XSS-adjacent minimal rule (Go)**: unsafe HTML responses (e.g., `text/template` for HTML, direct writes of untrusted input to `http.ResponseWriter`) with safe-pattern guidance.
* **Context index (RAG-lite)**: local index of main branch conventions (error handling, logging, function signatures) — **no external store**.
* **LLM abstraction**: drivers for `null`, OpenAI (GPT), Anthropic (Claude), DeepSeek (feature-gated).
* **Privacy defaults**: redaction of secrets in prompts, **no external calls by default**.

### Out of Scope (Post-MVP)

* GitHub/GitLab bot with inline comments.
* DAST, dependency/SCA, SBOM generation.
* Auto-patch application/PR updates.
* IDE extensions.

---

## 6) Epics, User Stories, Acceptance Criteria

### Epic A — CLI & Developer Experience

**A1. Install & Help**
*As a developer, I can install the CLI and see commands.*
**AC**: `reviewer --help` lists `check`, `index`, `print-config`, `version`.

**A2. Config**
*As a developer, I can configure models, budgets, and path filters.*
**AC**: `reviewer.toml` supports provider selection (`null|openai|anthropic|deepseek`), model name, token/budget caps, temperature, allow/deny paths, redaction on/off. `print-config` shows **effective config** and **compiled-in providers**. Precedence: **flags > env > file**.

**A3. CI Integration**
*As a maintainer, I can run the tool in CI and gate on findings.*
**AC**: GitHub Actions + GitLab CI templates; artifacts upload; exit codes (see §10).

### Epic B — Security & Quality Analysis (Rules-first)

**B1. Secrets Detection**
**AC**: Detect common key/token patterns; show location, severity, remediation tip.

**B2. SQL Injection (Go)**
**AC**: Flag `db.Query*`/`Exec` with concatenated inputs; suggest parameterization with snippet.

**B3. Unsafe Networking/Timeouts (Go)**
**AC**: Flag `http.Client{Timeout:0}` or missing contexts on outbound I/O; suggest safe patterns.

**B4. Noise Controls**
**AC**: Severity thresholds, path filters, per-rule toggles in config.

**B5. Server-side XSS-adjacent minimal**
**AC**: Flag HTML responses that use unescaped/unvetted user input (e.g., `text/template` for HTML, direct `fmt.Fprintf(w, user)`); suggest `html/template`, centralized sanitization, or encoding helpers.

### Epic C — Context-Aware Analysis (RAG-lite)

**C1. Index Main Branch**
**AC**: Store symbols/filenames/patterns locally at `.reviewer/index/` (versioned). Incremental updates.

**C2. Deviation Detection**
**AC**: Flag inconsistent error handling/logging/function signatures with concrete examples from repo.

### Epic D — Reporting & Summaries

**D1. PR Summary**
**AC**: First section includes purpose, scope, and key changes.

**D2. Hotspots & Critical Files**
**AC**: Top-5 hotspots ranked by `risk = sev_w * findings_in_file + churn_w * changed_lines` (defaults `sev_w=3`, `churn_w=1`; configurable).

**D3. Suggested Fixes**
**AC**: Each high-severity finding includes a minimal diff and rationale.

**D4. Diagrams (stubbed)**
**AC**: Mermaid sequence **stubs** based on diff context (no heavy call-graph), included when ≥3 hops inferred.

---

## 7) Non-Functional Requirements

* **Performance**: P50 ≤ 5m; P95 ≤ 8m on baseline; reuse index; parallel where safe.
* **Accuracy**: ≤ 15% FP (per-finding) on fixtures; bias to precision over recall.
* **Privacy & Security**: default `provider=null`; detection precedes redaction; redact only in **prompts and public report**, never in rules inputs.
* **Determinism**: CI mode pins model IDs; `temperature=0`; report logs model, driver, and ruleset versions.
* **Portability**: macOS (arm64/x86\_64) & Linux (arm64/x86\_64). Windows post-MVP if demanded.
* **Resource envelope**: P95 **RAM ≤ 1.5 GB**; CPU uses all cores by default (configurable).
* **Observability (local)**: verbose mode for debugging; **no telemetry by default** (opt-in).

---

## 8) Architecture Overview (MVP)

### Components

* **CLI (`reviewer`)** — args parsing, config loading, orchestrates a review run.
* **Engine (library)** — diff ingestion → rules → (optional) LLM → report.
* **Ruleset (rules-go)** — SAST-lite checks for Go + generic patterns.
* **RAG-lite Indexer** — local symbol/pattern index from base branch.
* **LLM Drivers** — `null`, OpenAI (GPT), Anthropic (Claude), DeepSeek (feature-gated crates).
* **Report Generator** — Markdown builder + Mermaid helpers.

**Sequence (happy path)**

1. Load config; resolve provider (defaults `null`).
2. Discover diff vs upstream base (auto); override with `--base-ref`.
3. Run rule checks on changed hunks (line-scoped findings).
4. Query RAG-lite index for conventions/examples as needed.
5. (Optional) Summarize and propose diffs via LLM driver (with redaction).
6. Generate `review_report.md`; print stdout summary; set exit code.

**Index location**: `.reviewer/index/` (versioned). Add `.reviewer/` to `.gitignore`.

---

## 9) Configuration (Schema, Precedence & Example)

**Precedence**: **flags > environment > file**.
**Common env vars**: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `DEEPSEEK_API_KEY`.

**Key Fields**

* `llm.provider`: `null | openai | anthropic | deepseek`
* `llm.model`: string
* `budget.tokens.max_per_run`: int
* `generation.temperature`: float (default low in CI)
* `privacy.redaction.enabled`: bool
* `paths.allow` / `paths.deny`: globs
* `rules`: per-rule enable/disable; severity thresholds
* `index.path`: default `.reviewer/index/`

**Example: `reviewer.toml`**

```toml
[llm]
provider = "null"              # default offline; set to openai|anthropic|deepseek to enable
model = "gpt-4o-mini"          # example; ignored when provider=null

[budget.tokens]
max_per_run = 200000

[generation]
temperature = 0.1

[privacy.redaction]
enabled = true
patterns = ["(?i)aws_?secret", "(?i)api[_-]?key", "(?i)token="]

[paths]
allow = ["src/**", "cmd/**"]
deny  = ["**/node_modules/**", "**/vendor/**", "**/.git/**"]

[rules]
secrets = { enabled = true, severity = "high" }
sql_injection_go = { enabled = true, severity = "critical" }
http_timeouts_go = { enabled = true, severity = "medium" }
server_xss_go = { enabled = true, severity = "medium" }
convention_deviation = { enabled = true, severity = "low" }

[index]
path = ".reviewer/index"
```

`print-config` must display: effective values, **compiled-in driver set**, and the resolved base ref.

---

## 10) Reporting (Markdown) & Exit Codes

**Sections**

1. **Summary** — purpose, scope, risk assessment.
2. **Security Findings** — high → low, each with: description, impact, location, minimal diff, references.
3. **Code Quality & Conventions** — deviations + concrete examples from repo.
4. **Hotspots** — top-5 files/lines + ranking rationale.
5. **Diagrams** — Mermaid sequence **stubs** for complex flows.
6. **Appendix** — sanitized config snapshot, run metadata (ruleset version, model/driver IDs, timings, warm/cold index).

**Exit Codes**

* `0` — No findings ≥ fail-level.
* `1` — Findings ≥ configured fail-level.
* `2` — Config/usage error.
* `3` — Runtime error (scan failed).

---

## 11) Testing & Evaluation

**Golden Fixtures**

* `fixtures/secrets/` (true positive leaks)
* `fixtures/sql-injection/` (unsafe concat)
* `fixtures/http-timeout/` (missing ctx/timeout)
* `fixtures/server-xss/` (unsafe HTML writes/minimal cases)
* `fixtures/clean/` (control)

**Harness**

* `make eval` runs CLI on fixtures; outputs runtime, counts, precision/FPs, report size, resource use.

**Targets**

* Precision ≥ **85%** (security findings) on fixtures.
* Runtime: **P50 ≤5m**, **P95 ≤8m** (baseline).
* Memory: **≤1.5 GB P95**.

---

## 12) Privacy, Security & Compliance

* **Default offline** (`provider = null`). No source code sent externally unless explicitly enabled.
* **Redaction order**: **detect → redact**; redaction applies only to LLM prompts and public report.
* **Logs**: local only; telemetry **opt-in** with clear docs.
* **Supply chain**: pinned Rust toolchain; `cargo-deny` for deps; `cargo audit`; release SBOM (optional post-MVP).
* **License**: **Apache-2.0**.
* **Security policy**: `SECURITY.md` with coordinated disclosure contact.

---

## 13) Risks & Mitigations

* **LLM hallucinations / bad advice** → rules-first pipeline; low temperature; require diffs with rationale; human-in-the-loop.
* **Provider outages / rate limits** → multi-provider fallback; budgets; retries with backoff; cached summaries.
* **Noise / alert fatigue** → conservative defaults, thresholds, per-rule toggles, diff-only scanning.
* **Performance on large repos** → incremental indexing; parallel scanning; path scoping; budgets.
* **Scope creep (call-graphs/XSS depth)** → keep XSS rule minimal and diagrams stubbed for MVP.

---

## 14) Release Plan & Milestones

* **0.1.0 (Week 1)**: Workspace scaffold; config; rules: secrets, SQLi-Go, HTTP timeouts; report; CI examples.
* **0.2.0 (Week 3)**: RAG-lite deviation checks; summaries; perf tuning.
* **0.3.0 (Week 5)**: Optional LLM integration (GPT/Claude/DeepSeek) with redaction; evaluation harness baseline.
* **0.4.0 (Week 7)**: Pilot in 3–5 repos; collect feedback/metrics.
* **0.5.0 (Week 9)**: Stabilization, docs; pre-release towards bot integration.
* **1.0.0-RC (Week 11)**:

  * Binaries for macOS (arm64/x86\_64) & Linux (arm64/x86\_64) with **SHA256** and **cosign** signatures.
  * **Homebrew tap** and **Docker image** for CI.
  * Docs complete (Quickstart, Config, Rule reference, CI setup, Troubleshooting).
  * **RC gates met** (see §15).
* **1.0.0-GA (Week 12)**: Tag after pilots confirm metrics; only critical fixes between RC and GA.

---

## 15) Release Gates (must be true to cut 1.0.0-RC)

**Functionality**

* Secrets, SQLi-Go, HTTP-timeouts, minimal server-XSS rules implemented with per-rule tests & docs.
* RAG-lite conventions (error/logging/signature) enforced with concrete examples.
* Reports contain all sections; CI templates exist; exit codes 0/1/2/3 implemented.

**Quality & Perf**

* Fixture results: precision ≥85%; FP rate ≤15%; P50 ≤5m, P95 ≤8m on baseline; memory ≤1.5 GB P95.

**Determinism/Privacy**

* CI mode pins model IDs; `temperature=0`; report includes model/driver/ruleset versions.
* Default `provider=null`; prompts redacted; no external calls unless enabled.

**DX/Distribution**

* Signed release artifacts + checksums; Homebrew tap; Docker image.
* `reviewer --help` accurate; `print-config` shows effective config + compiled-in providers.

**Docs/Policy**

* LICENSE (Apache-2.0), SECURITY.md, CONTRIBUTING.md, CODE\_OF\_CONDUCT.md.
* Quickstart, Rule reference, Config, CI setup, Troubleshooting.

**Pilot Evidence**

* Ran in ≥3 pilot repos for ≥2 weeks; usefulness ≥4/5; ≥2 critical issues blocked pre-merge.

---

## 16) Appendix

### A. OWASP Mapping (MVP focus)

* Injection (SQLi), Sensitive Data Exposure (secrets), Security Misconfiguration (timeouts), XSS-adjacent (minimal server-side patterns), Broken Access Control (post-MVP).

### B. Commands (CLI)

* `reviewer check [--path PATH] [--diff REF] [--base-ref REF] [--fail-on {critical,high,medium,low}]`
* `reviewer index [--path PATH]`
* `reviewer print-config`
* `reviewer version`

### C. Definitions

* **RAG-lite**: local, lightweight index of files/symbols/conventions — no external vector store.
* **SAST-lite**: curated, high-precision static checks focused on changed code.

### D. Distribution & Trust

* GitHub Releases (macOS/Linux, arm64/x86\_64), **cosign** signatures, **SHA256** checksums.
* **Homebrew tap** for macOS; **Docker image** for CI usage.

### E. Repo Hygiene

* Default index path: `.reviewer/index/`; add `.reviewer/` to `.gitignore`.

---

**This PRD is approved for 1.0.0-RC once §15 gates are satisfied.**