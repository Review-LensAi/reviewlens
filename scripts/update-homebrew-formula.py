#!/usr/bin/env python3
import sys
import pathlib
import subprocess
import urllib.request

if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} <version>", file=sys.stderr)
    sys.exit(1)

version = sys.argv[1]
repo = "Review-LensAi/reviewlens"
formula = pathlib.Path(__file__).resolve().parent.parent / "homebrew-tap" / "reviewlens.rb"

targets = {
    "ARM64_MAC_SHA256": "aarch64-apple-darwin",
    "X86_64_MAC_SHA256": "x86_64-apple-darwin",
    "ARM64_LINUX_SHA256": "aarch64-unknown-linux-gnu",
    "X86_64_LINUX_SHA256": "x86_64-unknown-linux-gnu",
}

checksums = {}
for key, target in targets.items():
    url = f"https://github.com/{repo}/releases/download/v{version}/reviewlens-{target}.tar.gz.sha256"
    with urllib.request.urlopen(url) as resp:
        checksums[key] = resp.read().decode().split()[0]

content = formula.read_text()
content = content.replace('version "<VERSION>"', f'version "{version}"')
for key, checksum in checksums.items():
    content = content.replace(f"<{key}>", checksum)
formula.write_text(content)

# verify installation if Homebrew is available
if subprocess.call(["which", "brew"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
    subprocess.check_call(["brew", "install", "--formula", str(formula)])
    subprocess.check_call(["reviewlens", "--version"])
else:
    print("Homebrew not found; skipping install verification.", file=sys.stderr)
