# Evaluation Harness

The repository includes a small set of Go fixtures under `fixtures/` used to measure runtime and detection precision for common scenarios:

- `fixtures/secrets` – contains a hard-coded API key.
- `fixtures/sql-injection` – demonstrates unsafe string concatenation in a SQL query.
- `fixtures/http-timeout` – performs an HTTP request without a timeout.
- `fixtures/clean` – minimal program with no issues (control).

Run the harness with:

```bash
make eval
```

This command indexes each fixture, runs the CLI, and reports per-fixture runtimes along with an overall precision metric.

The harness uses release builds to ensure accurate runtime and memory metrics.

## Latest Results

Executed `scripts/eval.sh` against a release build in a clean environment. Results are compared with PRD thresholds.

| Metric | Result | Threshold | Status |
| --- | --- | --- | --- |
| Precision | 1.00 | ≥ 0.85 | Pass |
| FP Rate | 0.00 | ≤ 0.15 | Pass |
| Runtime P50 (ms) | 394 | ≤ 25000 | Pass |
| Runtime P95 (ms) | 418 | ≤ 60000 | Pass |
| Memory P50 (KB) | 59680 | – | n/a |
| Memory P95 (KB) | 59744 | ≤ 1572864 | Pass |

All evaluated metrics meet the PRD performance and quality targets.
