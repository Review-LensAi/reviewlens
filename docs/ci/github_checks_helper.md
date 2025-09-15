# GitHub Checks helper

The `scripts/ci_helper.py` script runs `reviewlens check` in a CI
environment and reports the results to the GitHub Checks API. When
invoked with `--allow-suggest`, the helper also submits diff suggestions
as pull request review comments.

## Usage

```sh
python scripts/ci_helper.py --diff origin/main --allow-suggest
```

The script requires the standard GitHub Actions environment variables:

- `GITHUB_TOKEN`
- `GITHUB_REPOSITORY`
- `GITHUB_SHA`
- `GITHUB_REF` (used to detect the pull request number)

It writes `review_report.json` to the working directory and exits with
the same status code as `reviewlens check`, ensuring CI reflects the
review outcome.
