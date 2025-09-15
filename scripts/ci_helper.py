#!/usr/bin/env python3
"""Run reviewlens in CI and post results to GitHub.

This helper executes `reviewlens check` with JSON output, publishes a
summary to the GitHub Checks API, and optionally posts diff suggestions as
review comments when `--allow-suggest` is used.
"""

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.request


def diff_to_suggestion(diff: str) -> str:
    """Convert a unified diff snippet to a GitHub suggestion block."""
    lines = []
    for line in diff.splitlines():
        if line.startswith("+") and not line.startswith("+++"):
            lines.append(line[1:])
    return "\n".join(lines)


def api_request(url: str, token: str, data: dict) -> None:
    body = json.dumps(data).encode()
    req = urllib.request.Request(
        url,
        data=body,
        headers={
            "Authorization": f"token {token}",
            "Accept": "application/vnd.github+json",
        },
        method="POST",
    )
    with urllib.request.urlopen(req) as resp:
        resp.read()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--allow-suggest", action="store_true", help="post diff suggestions")
    parser.add_argument("--path", default=".", help="repository path")
    parser.add_argument("--diff", default="auto", help="base ref for diff")
    args = parser.parse_args()

    cmd = [
        "reviewlens",
        "check",
        "--ci",
        "--format",
        "json",
        "--output",
        "review_report.json",
        "--path",
        args.path,
        "--diff",
        args.diff,
    ]
    if args.allow_suggest:
        cmd.append("--allow-suggest")

    result = subprocess.run(cmd)
    exit_code = result.returncode

    try:
        with open("review_report.json", "r", encoding="utf-8") as f:
            report = json.load(f)
    except FileNotFoundError:
        return exit_code

    repo = os.getenv("GITHUB_REPOSITORY")
    sha = os.getenv("GITHUB_SHA")
    token = os.getenv("GITHUB_TOKEN")
    if not all([repo, sha, token]):
        return exit_code

    checks_url = f"https://api.github.com/repos/{repo}/check-runs"
    conclusion = "success" if exit_code == 0 else "failure"
    check_payload = {
        "name": "reviewlens",
        "head_sha": sha,
        "status": "completed",
        "conclusion": conclusion,
        "output": {
            "title": "ReviewLens Report",
            "summary": report.get("summary", ""),
        },
    }
    api_request(checks_url, token, check_payload)

    if args.allow_suggest:
        ref = os.getenv("GITHUB_REF", "")
        m = re.match(r"refs/pull/(\d+)/", ref)
        pr_number = m.group(1) if m else os.getenv("PR_NUMBER")
        if pr_number:
            comments_url = f"https://api.github.com/repos/{repo}/pulls/{pr_number}/comments"
            for issue in report.get("issues", []):
                diff = issue.get("diff")
                if not diff:
                    continue
                suggestion = diff_to_suggestion(diff)
                if not suggestion.strip():
                    continue
                comment = {
                    "body": f"```suggestion\n{suggestion}\n```",
                    "commit_id": sha,
                    "path": issue.get("file_path"),
                    "side": "RIGHT",
                    "line": issue.get("line_number", 1),
                }
                api_request(comments_url, token, comment)

    return exit_code


if __name__ == "__main__":
    sys.exit(main())

