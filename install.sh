#!/bin/bash
# install.sh — Install Zeta compiler and zorb CLI
# Usage: curl -sSf https://raw.githubusercontent.com/murphsicles/zeta/main/install.sh | sh
# Or:   curl -sSf https://get.zetac.io | sh
#
# Downloads latest zetac binary for your platform and installs to /usr/local/bin.
# Supports: Linux x86_64

set -euo pipefail

ZETA_VERSION="v1.0.18"
ZETA_REPO="murphsicles/zeta"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="zetac"
RELEASE_URL="https://github.com/${ZETA_REPO}/releases/download/${ZETA_VERSION}"

# Colors
BOLD='\033[1m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo -e "${BOLD}${CYAN}═══ Zeta Compiler Installer ═══${NC}"
echo ""

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
    Linux-x86_64|Linux-amd64)
        TARGET="zetac-linux-x64"
        ;;
    *)
        echo -e "${RED}Unsupported platform: ${OS} ${ARCH}${NC}"
        echo "Currently supported: Linux x86_64"
        echo "Build from source: https://github.com/${ZETA_REPO}"
        exit 1
        ;;
esac

DOWNLOAD_URL="${RELEASE_URL}/${TARGET}"

# Check if already installed
if command -v "${BINARY_NAME}" &>/dev/null; then
    INSTALLED_VER="$(${BINARY_NAME} --version 2>/dev/null || echo "unknown")"
    echo -e "  Found: ${BINARY_NAME} (${INSTALLED_VER})"
    echo -e "  Target: ${ZETA_VERSION}"
    echo ""
    echo -n "Overwrite? [y/N] "
    read -r CONFIRM
    if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
        echo "Aborted."
        exit 0
    fi
fi

# Check write permission
if [ ! -w "${INSTALL_DIR}" ]; then
    echo -e "${RED}Cannot write to ${INSTALL_DIR} — need sudo?${NC}"
    echo "Trying with sudo..."
    INSTALL_DIR="/usr/local/bin"
    SUDO="sudo"
else
    SUDO=""
fi

# Download
echo -e "  Downloading ${BOLD}${BINARY_NAME}${NC} ${ZETA_VERSION}..."
TMPFILE=$(mktemp)
trap 'rm -f "${TMPFILE}"' EXIT

if command -v curl &>/dev/null; then
    curl -sSfL "${DOWNLOAD_URL}" -o "${TMPFILE}"
elif command -v wget &>/dev/null; then
    wget -q "${DOWNLOAD_URL}" -O "${TMPFILE}"
else
    echo -e "${RED}Need curl or wget to download.${NC}"
    exit 1
fi

# Verify it's an executable (not HTML error page)
if ! file "${TMPFILE}" | grep -q "ELF\|executable"; then
    echo -e "${RED}Download failed — got non-binary response.${NC}"
    echo "  URL: ${DOWNLOAD_URL}"
    echo "  Try manually: https://github.com/${ZETA_REPO}/releases"
    exit 1
fi

# Install
chmod +x "${TMPFILE}"
${SUDO} mv "${TMPFILE}" "${INSTALL_DIR}/${BINARY_NAME}"

echo -e "  Installed to: ${GREEN}${INSTALL_DIR}/${BINARY_NAME}${NC}"

# Verify
if command -v "${BINARY_NAME}" &>/dev/null; then
    echo ""
    echo -e "${GREEN}${BOLD}✓ Zeta ${ZETA_VERSION} installed successfully!${NC}"
    echo ""
    echo "  Quick start:"
    echo "    zetac --help           # Show compiler options"
    echo "    zetac --zorb search    # Search zorbs.io packages"
    echo "    zetac --zorb add @crypto/nour"
    echo "    zetac hello.z          # Compile a Zeta file"
    echo ""
    echo "  Documentation:  https://zorbs.io/docs"
else
    echo -e "${RED}Installation failed — ${INSTALL_DIR} not in PATH?${NC}"
    echo "  Try adding to your shell profile:"
    echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
    exit 1
fi
