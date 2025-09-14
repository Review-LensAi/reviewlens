# Evaluation Harness

The repository includes a small set of Go fixtures under `fixtures/` used to measure runtime and detection precision for common scenarios:

- `fixtures/secrets` – contains a hard-coded API key.
- `fixtures/sql-injection` – demonstrates unsafe string concatenation in a SQL query.
- `fixtures/http-timeout` – performs an HTTP request without a timeout.
- `fixtures/server-xss` – writes unsanitized user input to an HTTP response.
- `fixtures/clean` – minimal program with no issues (control).

Run the harness with:

```bash
make eval
```

This command indexes each fixture, runs the CLI, and reports per-fixture runtimes along with an overall precision metric.

The harness uses release builds to ensure accurate runtime and memory metrics.
