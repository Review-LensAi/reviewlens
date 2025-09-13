# Reviewlens

Reviewlens is a CLI-based MVP for context-aware, security-first code reviews that summarizes changes and flags issues.

## Quick Start

Install:

```bash
pip install reviewlens
```

Run:

```bash
reviewlens check --path . --diff main
```

## Non-vendor lock

Supports GPT, Claude, DeepSeek, and local models.

## What the MVP won't do

- GitHub/GitLab inline comments
- DAST
- SCA
- IDE plugins
- Auto-commits
