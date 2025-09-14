# Release checklist

1. Build and upload release artifacts for all supported targets.
2. Ensure each artifact includes a corresponding `.sha256` file.
3. Update the Homebrew formula by running:
   ```bash
   python scripts/update-homebrew-formula.py <version>
   ```
   This script pulls checksums from the release artifacts and updates `homebrew-tap/reviewlens.rb`.
4. Validate the formula on macOS and Linux:
   ```bash
   brew install --build-from-source homebrew-tap/reviewlens.rb
   reviewlens --version
   ```
   Run the command on both macOS and Linux and confirm the reported version matches the release tag.
5. Commit the updated formula and proceed with tagging the release.
