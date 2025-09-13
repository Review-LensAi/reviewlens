Project Summary — Intelligent Code Review, Debugging & Security Agent (CLI-first)
Vision (North Star)
Make code review 10× faster and safer by putting an expert AI reviewer directly in the developer loop—local, CI, and PR—catching critical issues early and explaining fixes clearly.
Problem We’re Solving
PRs are bigger (AI-assisted coding), human reviewers are overloaded, and security flaws slip through.
Existing tools skew passive (linters/scanners) and lack deep repo context or actionable, teachable feedback.
Core Hypothesis
An AI agent with deep codebase context (RAG) + targeted security analysis (SAST-lite) can consistently surface high-impact issues and cut review time from hours to minutes—without spamming devs.
Target Users
Priya (Senior Reviewer): wants high-signal highlights and architectural red flags.
Leo (Junior Submitter): wants immediate, concrete feedback and fixes.
MVP Scope (CLI-first)
Modes
check (local/CI): analyze working tree or --diff main.
Outputs: Markdown report (stdout + review_report.md) with:
Security findings (top OWASP 5), 2) Context-aware code quality deviations,
High-level PR summary, 4) Suggested fixes, 5) Optional sequence diagram.
Capabilities
SAST-lite: detect SQLi, XSS, command injection, insecure deserialization, SSRF patterns (language-aware where possible).
Context (RAG-lite): index main branch; detect deviations (error handling, logging, layering, naming).
Summarization: natural-language overview + “files to focus” + sequence diagram for complex flows.
CI-friendly: deterministic exit codes (0 clean, 1 issues, 2 internal error) and JSON export flag for future bot wrappers.
Initial Language Focus (suggested)
JS/TS, Python, Go (extensible rules; language-agnostic heuristics for everything else).
Non-Functional MVP Targets
Performance: first report within ≤5 minutes for typical PRs (<1k added LOC).
Accuracy: <15% false positive rate on security flags (tracked via suppression feedback).
Usability: comments and suggestions are copy-paste-able, minimal jargon.
Success Metrics (MVP)
Adoption: ≥70% of PRs in pilot repos run the CLI in CI.
Speed: median time-to-first-feedback ≤3 minutes in CI.
Signal: ≥60% of surfaced issues acknowledged (fixed or justified).
Quality: false-positive rate <15% on security rules, <20% overall.
Value: reviewer time per PR reduced by ≥30% (self-reported).
Out of Scope (Post-MVP)
GitHub/GitLab inline bot + Checks API (will wrap CLI next).
DAST, dependency scanning, SBOM.
Auto-commit fixes via bot.
Multi-repo monorepo graph analysis and cross-service diff impact.
Product Principles
Local-first & privacy-aware: default to running locally/CI; no code leaves environment unless configured.
Explain, don’t just alert: every finding includes why it matters + safe fix.
Low noise, high signal: rank by risk; collapse nits; deduplicate across files.
Composable: core engine is a library; CLI and integrations are thin wrappers.
Risks & Mitigations
Noise risk: start narrow (top OWASP), strict rule gating, allow inline # reviewer:ignore with audit trail.
Perf on large repos: cache index, incremental diffs, parallel scanning.
Language drift: rules plugin system; community rule packs.
Trust: dry-run mode, clear provenance for suggestions.
Deliverables (MVP)
reviewer-core (Rust lib): indexing, analyzers, ranking, renderers.
reviewer CLI: flags, IO, exit codes.
Report schema: Markdown + optional JSON.
Starter rule packs: security + conventions for JS/TS, Python, Go.
Docs: quickstart, CI examples, suppression/ignore policy, contribution guide.
Sample repos for demos and regression tests.
Definition of Done (CLI MVP)
Runs locally and in GitHub Actions/GitLab CI with one line.
Generates actionable review_report.md covering: summary, top risks, line refs, fixes, diagram (when applicable).
Stable exit codes; non-zero on high/critical issues.
Config file: include/exclude paths, rule toggles, severity thresholds.
Benchmarked on 3 real OSS repos; metrics published in README.
Near-Term Roadmap (pragmatic)
Week 1–2: workspace setup, core library API, repo indexer, basic diff, Markdown renderer.
Week 3–4: OWASP-5 detectors (language-aware), ranking, config, JSON export, perf pass.
Week 5: RAG-lite conventions detector, PR summary, sequence diagram generator.
Week 6: CI templates, docs, sample repos, public alpha.
