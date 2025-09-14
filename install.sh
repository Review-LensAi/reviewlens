#!/bin/sh

# This script installs the reviewlens for Linux and macOS.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Review-LensAi/reviewlens/main/install.sh | sh
#
# The script will:
# 1. Detect the user's OS and architecture.
# 2. Download the latest release from GitHub.
# 3. Unpack the binary.
# 4. Move the binary to /usr/local/bin (may require sudo).

set -e # Exit on error

# --- Configuration ---
REPO="Review-LensAi/reviewlens"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="reviewlens"

# --- Helper Functions ---
print_info() {
    echo "\033[1;34m[INFO]\033[0m $1"
}

print_error() {
    echo "\033[1;31m[ERROR]\033[0m $1" >&2
    exit 1
}

# --- Main Script ---
print_info "Starting installation of ${BINARY_NAME}..."

# 1. Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    Darwin)
        TARGET_OS="apple-darwin"
        ;;
    *)
        print_error "Unsupported OS: ${OS}. Only Linux and macOS are supported."
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        print_error "Unsupported architecture: ${ARCH}. Only x86_64 and aarch64 are currently supported."
        ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"
ASSET_NAME="${BINARY_NAME}-${TARGET}.tar.gz"

# 2. Get the latest release tag from GitHub API
print_info "Fetching the latest release tag..."
# We use curl -s to make it silent and grep/sed to parse the JSON response.
# This avoids needing a tool like `jq`.
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    print_error "Could not fetch the latest release tag. Check the repository URL and your connection."
fi

print_info "Latest release is ${LATEST_TAG}."

# 3. Download the release asset
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ASSET_NAME}"
DOWNLOAD_DIR=$(mktemp -d)

print_info "Downloading from ${DOWNLOAD_URL}..."
curl -L -o "${DOWNLOAD_DIR}/${ASSET_NAME}" "$DOWNLOAD_URL"

# Download checksum and signature files
print_info "Fetching checksum and signature..."
curl -L -o "${DOWNLOAD_DIR}/${ASSET_NAME}.sha256" "${DOWNLOAD_URL}.sha256"
curl -L -o "${DOWNLOAD_DIR}/${ASSET_NAME}.sig" "${DOWNLOAD_URL}.sig"

# Verify checksum
print_info "Verifying checksum..."
EXPECTED_SHA256=$(cut -d ' ' -f1 "${DOWNLOAD_DIR}/${ASSET_NAME}.sha256")
if ! echo "${EXPECTED_SHA256}  ${DOWNLOAD_DIR}/${ASSET_NAME}" | sha256sum -c - >/dev/null 2>&1; then
    print_error "Checksum verification failed"
fi

# Verify signature if cosign is available
if command -v cosign >/dev/null 2>&1; then
    print_info "Verifying signature with cosign..."
    if ! cosign verify-blob --signature "${DOWNLOAD_DIR}/${ASSET_NAME}.sig" "${DOWNLOAD_DIR}/${ASSET_NAME}" >/dev/null 2>&1
    then
        print_error "Cosign signature verification failed"
    fi
else
    print_info "cosign not found; skipping signature verification."
fi

# 4. Unpack the binary
print_info "Unpacking the binary..."
tar -xzf "${DOWNLOAD_DIR}/${ASSET_NAME}" -C "${DOWNLOAD_DIR}"

# 5. Install the binary
INSTALL_PATH="${INSTALL_DIR}/${BINARY_NAME}"
print_info "Installing to ${INSTALL_PATH}..."

# Check if the install directory is writable. If not, use sudo.
if [ -w "$INSTALL_DIR" ]; then
    mv "${DOWNLOAD_DIR}/${BINARY_NAME}" "$INSTALL_PATH"
else
    print_info "Sudo privileges are required to install to ${INSTALL_DIR}."
    sudo mv "${DOWNLOAD_DIR}/${BINARY_NAME}" "$INSTALL_PATH"
fi

chmod +x "$INSTALL_PATH"

# 6. Clean up
rm -rf "$DOWNLOAD_DIR"

print_info "${BINARY_NAME} has been installed successfully!"
print_info "You can now run '${BINARY_NAME}' from your terminal."
