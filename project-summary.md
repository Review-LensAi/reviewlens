Intelligent Code Review Agent — Project Goals (MVP)
Vision
A context-aware, security-first code review agent that runs locally or in CI, summarizes changes, flags real issues (not noise), and suggests precise fixes—without vendor lock-in to any single LLM.
Primary Objectives
Slash review time while improving quality and security.
Catch high-impact issues early: OWASP Top 5, dangerous patterns, secret/credential leaks.
Understand codebase context (RAG) to give suggestions that match project conventions.
Deliver actionable output: concise PR summary, hotspots, concrete diffs/suggestions.
Be provider-agnostic: support GPT, Claude, DeepSeek, and self-hosted models behind a clean abstraction.
Non–Vendor-Locked LLM Strategy
Provider-agnostic interface: a single LLM “driver” trait + adapters (OpenAI, Anthropic, DeepSeek, OpenRouter/compatible, self-hosted via vLLM/TGI/LiteLLM).
Config over code: choose model/provider via reviewer.toml / env vars; easy multi-provider fallback & routing.
Prompt & tool schema versioning: versioned prompts/templates, function-calling/tool schemas kept model-neutral.
Offline/air-gapped mode: allow pure static analysis + optional local model usage (no code leaves the machine).
Cost/latency controls: max tokens, temperature, retries, circuit breakers, and per-provider budgets.
What We’re Building First (CLI-First MVP)
CLI command: reviewlens check --path . --diff main
Outputs:
review_report.md (summary, risks, suggested fixes, Mermaid diagrams)
Exit codes for CI gating (0 clean, 1 issues)
Core capabilities:
Security scan (SAST-lite): flag top OWASP families + secrets/credentials patterns.
Context-aware checks (RAG-lite): index main branch, detect pattern deviations (error handling, logging, boundary checks, API usage).
PR summary & hotspots: what changed, why it matters, where to focus.
Fix suggestions: minimal, reviewable diffs with rationale.
Performance guardrail: first pass finishes < 5 minutes on typical PRs.
Quality guardrail: <15% false positive rate target on curated test PRs.
Out of Scope for MVP (Post-MVP)
GitHub/GitLab inline auto-comments & Checks API (we’ll post reports first via CI; bot later).
DAST, full dependency SCA, and wide multi-language coverage (start with 1–2 languages + generic patterns).
Autonomous code changes/auto-commit (gated for later once trust is established).
IDE plugins (after CLI is solid).
Success Metrics (MVP)
Adoption: # repos running CLI in CI.
Time to feedback: P50 end-to-end under 5 minutes.
Signal quality: Reviewer “useful” ratings ≥ 4/5; false positives ≤ 15%.
Defect catch rate: % of issues caught pre-merge vs baseline.
Security value: # critical/security issues blocked pre-merge.
Architecture Principles
Separation of concerns: engine (library) ⟷ cli (thin wrapper) ⟷ integrations (actions/bots later).
Pluggable providers: LLM, embeddings, vector store (e.g., Tantivy/Qdrant), scanners (Semgrep-style, regex/rules).
Deterministic CI mode: pinned model versions, low temperature, reproducible prompts.
Privacy & safety by default:
Path allowlists/denylists, redaction of secrets before LLM calls, optional “no external calls” mode.
Local cache for embeddings/index; clear data-retention story.
Initial Deliverables (Phase 1)
Docs: README with goals, quick start, config examples.
Configs: reviewer.toml (model, provider, thresholds, redaction rules).
Engine:
Diff ingestion + changed-code focus
RAG-lite indexer (main branch)
Security & pattern rules (extensible ruleset)
Report generator (Markdown + Mermaid)
LLM abstraction + providers (GPT, Claude, DeepSeek + a “null” local mode)
CLI: check, index, print-config, version
CI examples: GitHub Action & GitLab CI templates (run CLI, upload artifact)
Near-Term Roadmap
MVP CLI (local & CI) → validate core hypothesis.
Report posting via GitHub Action/GitLab job (paste Markdown to PR).
Bot/Checks integration (inline comments, status checks).
Remediation mode (optional patch generation with guardrails).
Language expansion & richer rulesets; eval harness & benchmark suite.
Risks & Mitigations
Hallucinations / bad fixes → require diffs + rationale, keep temperature low, rule-first then LLM-assist, human-in-the-loop.
Provider outages / rate limits → multi-provider fallback, caching, backoff, budget limits.
Leakage/privacy → redaction, allowlists, local-only mode, explicit consent in CI.
Noise → tight ruleset, confidence thresholds, “only changed lines” focus, continuous eval against golden PRs.
