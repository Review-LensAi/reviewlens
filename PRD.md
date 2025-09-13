# ReviewLens Product Requirements Document (PRD)

## Vision
A context-aware, security-first code review agent that runs locally or in CI, summarizing changes, flagging real issues, and suggesting fixes without vendor lock-in.

## Primary Objectives
- Slash review time while improving quality and security.
- Catch high-impact issues early (OWASP Top 5, dangerous patterns, secret or credential leaks).
- Understand codebase context via RAG to give suggestions matching project conventions.
- Deliver actionable output such as concise PR summaries, hotspots, and concrete diffs.
- Remain provider-agnostic across GPT, Claude, DeepSeek, and self-hosted models.

## MVP Scope

### Core Capabilities
- CLI usage: `reviewer check --path . --diff main`.
- Security scans (SAST-lite) for top OWASP families and secret/credential patterns.
- RAG-lite indexing of the main branch to detect pattern deviations.
- Report generation: produces `review_report.md` with summaries, risks, suggested fixes, and diagrams.

### Success Metrics
- Adoption: number of repositories running the CLI in CI.
- Time to feedback: median end-to-end runtime under 5 minutes.
- Signal quality: useful ratings ≥ 4/5 and false positives ≤ 15%.
- Defect catch rate: percentage of issues caught pre-merge vs baseline.
- Security value: count of critical/security issues blocked pre-merge.

## Out of Scope
- GitHub/GitLab integration for inline comments or Checks API.
- Dynamic application security testing (DAST) and full dependency SCA.
- IDE plugins.
- Autonomous code changes or auto-commits.

## Future Enhancements
- GitHub/GitLab integration for smoother PR/MR review workflows.
