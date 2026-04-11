#!/bin/sh
# Pastel installer — https://github.com/AgentsMesh/pastel
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/AgentsMesh/pastel/main/scripts/install.sh | sh
#   curl -fsSL ... | sh -s -- --nightly    # install nightly snapshot

set -e

REPO="AgentsMesh/pastel"
BINARY="pastel"
CHANNEL="stable"  # or "nightly"

# Parse args
for arg in "$@"; do
    case "$arg" in
        --nightly) CHANNEL="nightly" ;;
    esac
done

main() {
    detect_platform
    fetch_latest_version
    download_and_install
    verify_install
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  PLATFORM="linux" ;;
        Darwin) PLATFORM="darwin" ;;
        *)
            echo "Error: unsupported OS: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_TAG="x86_64" ;;
        aarch64|arm64)   ARCH_TAG="aarch64" ;;
        *)
            echo "Error: unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    TARGET="${BINARY}-${PLATFORM}-${ARCH_TAG}"
    echo "Detected platform: ${OS} ${ARCH} -> ${TARGET}"
}

fetch_latest_version() {
    if [ "$CHANNEL" = "nightly" ]; then
        VERSION="nightly"
        echo "Using nightly snapshot"
        return
    fi

    echo "Fetching latest release..."
    VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"

    if [ -z "$VERSION" ]; then
        echo "Error: could not determine latest version"
        exit 1
    fi

    echo "Latest version: ${VERSION}"
}

download_and_install() {
    if [ "$CHANNEL" = "nightly" ]; then
        # Nightly uses date-based naming
        ARCHIVE="pastel-nightly-*-${TARGET}.tar.gz"
        # List assets from nightly release and pick the right one
        ASSET_URL="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/tags/nightly" \
            | grep '"browser_download_url"' \
            | grep "${TARGET}" \
            | grep -v sha256 \
            | head -1 \
            | sed -E 's/.*"browser_download_url": *"([^"]+)".*/\1/')"
        if [ -z "$ASSET_URL" ]; then
            echo "Error: no nightly build found for ${TARGET}"
            exit 1
        fi
        URL="$ASSET_URL"
    else
        ARCHIVE="${BINARY}-${VERSION}-${TARGET}.tar.gz"
        URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
    fi

    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT

    echo "Downloading ${URL}..."
    curl -fsSL "$URL" -o "${TMPDIR}/${ARCHIVE}"

    echo "Extracting..."
    tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

    # Determine install directory
    INSTALL_DIR="/usr/local/bin"
    if [ ! -w "$INSTALL_DIR" ]; then
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
        echo "Note: installing to ${INSTALL_DIR} (no write access to /usr/local/bin)"
        echo "      Make sure ${INSTALL_DIR} is in your PATH."
    fi

    mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
    echo "Installed to ${INSTALL_DIR}/${BINARY}"
}

verify_install() {
    if command -v "$BINARY" >/dev/null 2>&1; then
        echo ""
        echo "Pastel installed successfully!"
        "$BINARY" --version
    else
        echo ""
        echo "Pastel installed to ${INSTALL_DIR}/${BINARY}"
        echo "Run '${BINARY} --version' to verify (you may need to restart your shell)."
    fi
}

main
