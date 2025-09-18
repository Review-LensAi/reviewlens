#!/usr/bin/env python3
import sys
import pathlib
import subprocess
import urllib.request
from typing import Dict

if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} <version>", file=sys.stderr)
    sys.exit(1)

version = sys.argv[1]
repo = "Review-LensAi/reviewlens"
formula = pathlib.Path(__file__).resolve().parent.parent / "homebrew-tap" / "reviewlens.rb"

targets = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
)

checksums: Dict[str, str] = {}
for target in targets:
    url = f"https://github.com/{repo}/releases/download/v{version}/reviewlens-{target}.tar.gz.sha256"
    with urllib.request.urlopen(url) as resp:
        checksums[target] = resp.read().decode().split()[0]

lines = formula.read_text().splitlines()
updated_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    stripped = line.strip()

    if stripped.startswith("version "):
        indent = line[: len(line) - len(stripped)]
        updated_lines.append(f'{indent}version "{version}"')
        i += 1
        continue

    if 'url "https://github.com/Review-LensAi/reviewlens/releases/download/v\\#{version}/reviewlens-' in line:
        updated_lines.append(line)
        i += 1
        if i >= len(lines):
            raise SystemExit("Expected sha256 line after url")
        sha_line = lines[i]
        sha_stripped = sha_line.strip()
        if not sha_stripped.startswith("sha256 "):
            raise SystemExit("Expected sha256 line after url")

        replaced = False
        for target, checksum in checksums.items():
            if f"reviewlens-{target}.tar.gz" in line:
                indent = sha_line[: len(sha_line) - len(sha_stripped)]
                updated_lines.append(f'{indent}sha256 "{checksum}"')
                replaced = True
                break

        if not replaced:
            raise SystemExit(f"No checksum found for url: {line}")

        i += 1
        continue

    updated_lines.append(line)
    i += 1

formula.write_text("\n".join(updated_lines) + "\n")

# verify installation if Homebrew is available
if subprocess.call(["which", "brew"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
    subprocess.check_call(["brew", "install", "--formula", str(formula)])
    subprocess.check_call(["reviewlens", "--version"])
else:
    print("Homebrew not found; skipping install verification.", file=sys.stderr)
