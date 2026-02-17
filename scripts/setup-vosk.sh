#!/bin/bash
# Download and install the Vosk native library for local STT support.
#
# Usage:
#   ./scripts/setup-vosk.sh
#
# After running this script, rebuild with:
#   cd src-tauri && cargo build --features vosk-stt

set -euo pipefail

VOSK_VERSION="0.3.45"
VOSK_DIR="$(pwd)/src-tauri/vosk-lib"

# Detect platform
case "$(uname -s)-$(uname -m)" in
    Darwin-arm64)
        PLATFORM="osx"
        LIB_EXT="dylib"
        ;;
    Darwin-x86_64)
        PLATFORM="osx"
        LIB_EXT="dylib"
        ;;
    Linux-x86_64)
        PLATFORM="linux-x86_64"
        LIB_EXT="so"
        ;;
    Linux-aarch64)
        PLATFORM="linux-aarch64"
        LIB_EXT="so"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="win64"
        LIB_EXT="dll"
        ;;
    *)
        echo "Unsupported platform: $(uname -s)-$(uname -m)"
        exit 1
        ;;
esac

URL="https://github.com/alphacep/vosk-api/releases/download/v${VOSK_VERSION}/vosk-${PLATFORM}-${VOSK_VERSION}.zip"

echo "=== Vosk Native Library Setup ==="
echo "Platform: ${PLATFORM}"
echo "Version: ${VOSK_VERSION}"
echo "URL: ${URL}"
echo "Target: ${VOSK_DIR}"
echo ""

if [ -f "${VOSK_DIR}/libvosk.${LIB_EXT}" ]; then
    echo "Vosk library already installed at ${VOSK_DIR}"
    echo "To reinstall, delete ${VOSK_DIR} and run this script again."
    exit 0
fi

mkdir -p "${VOSK_DIR}"

echo "Downloading Vosk library..."
TEMP_ZIP="/tmp/vosk-${PLATFORM}-${VOSK_VERSION}.zip"
curl -L -o "${TEMP_ZIP}" "${URL}"

echo "Extracting..."
unzip -o "${TEMP_ZIP}" -d "/tmp/vosk-extract"

# Find and copy library files
EXTRACTED_DIR="/tmp/vosk-extract/vosk-${PLATFORM}-${VOSK_VERSION}"
if [ ! -d "${EXTRACTED_DIR}" ]; then
    # Try without version suffix
    EXTRACTED_DIR=$(find /tmp/vosk-extract -maxdepth 1 -type d -name "vosk-*" | head -1)
fi

if [ -z "${EXTRACTED_DIR}" ] || [ ! -d "${EXTRACTED_DIR}" ]; then
    echo "ERROR: Could not find extracted vosk directory"
    exit 1
fi

cp -v "${EXTRACTED_DIR}"/*.${LIB_EXT}* "${VOSK_DIR}/" 2>/dev/null || true
cp -v "${EXTRACTED_DIR}"/*.h "${VOSK_DIR}/" 2>/dev/null || true

# Clean up
rm -rf "/tmp/vosk-extract" "${TEMP_ZIP}"

echo ""
echo "=== Setup Complete ==="
echo "Vosk library installed to: ${VOSK_DIR}"
echo ""
echo "To build with Vosk support:"
echo "  export VOSK_PATH=${VOSK_DIR}"
echo "  cd src-tauri && cargo build --features vosk-stt"
echo ""
echo "Or add to your shell profile:"
echo "  echo 'export VOSK_PATH=${VOSK_DIR}' >> ~/.zshrc"
