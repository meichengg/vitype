#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/build_dmg.sh [--skip-build] [--app /path/to/ViType.app] [--out-dir /path/to/out] [--volname ViType]

Defaults:
  - Builds the Release app via xcodebuild (unless --skip-build is passed)
  - Output DMG is written to: <repo>/ViType-macos/dist/
  - DMG filename includes version read from the built app's Info.plist

Examples:
  cd ViType-macos
  ./scripts/build_dmg.sh

  ./scripts/build_dmg.sh --skip-build --app "/path/to/ViType.app"
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${PROJECT_DIR}/.." && pwd)"

SKIP_BUILD="0"
APP_PATH=""
OUT_DIR="${PROJECT_DIR}/dist"
VOLNAME="ViType"

while [ $# -gt 0 ]; do
  case "$1" in
    --skip-build)
      SKIP_BUILD="1"
      shift
      ;;
    --app)
      APP_PATH="${2:-}"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="${2:-}"
      shift 2
      ;;
    --volname)
      VOLNAME="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      echo >&2
      usage >&2
      exit 2
      ;;
  esac
done

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required command not found in PATH: $1" >&2
    exit 1
  fi
}

require_cmd hdiutil
require_cmd /usr/bin/ditto
require_cmd /usr/bin/plutil

if [ "${SKIP_BUILD}" != "1" ]; then
  require_cmd xcodebuild

  DERIVED_DATA="${PROJECT_DIR}/.derivedData"
  rm -rf "${DERIVED_DATA}"

  xcodebuild \
    -project "${PROJECT_DIR}/ViType.xcodeproj" \
    -scheme "ViType" \
    -configuration "Release" \
    -derivedDataPath "${DERIVED_DATA}" \
    build

  APP_PATH="${DERIVED_DATA}/Build/Products/Release/ViType.app"
fi

if [ -z "${APP_PATH}" ]; then
  echo "error: --app is required when using --skip-build" >&2
  exit 2
fi

if [ ! -d "${APP_PATH}" ]; then
  echo "error: app not found: ${APP_PATH}" >&2
  exit 1
fi

INFO_PLIST="${APP_PATH}/Contents/Info.plist"
if [ ! -f "${INFO_PLIST}" ]; then
  echo "error: Info.plist not found in app bundle: ${INFO_PLIST}" >&2
  exit 1
fi

APP_NAME="$(basename "${APP_PATH}" .app)"
VERSION="$(/usr/bin/plutil -extract CFBundleShortVersionString raw -o - "${INFO_PLIST}" 2>/dev/null || true)"
BUILD="$(/usr/bin/plutil -extract CFBundleVersion raw -o - "${INFO_PLIST}" 2>/dev/null || true)"

if [ -z "${VERSION}" ]; then
  VERSION="0.0.0"
fi
if [ -z "${BUILD}" ]; then
  BUILD="0"
fi

mkdir -p "${OUT_DIR}"

STAGING_DIR="$(mktemp -d "${TMPDIR:-/tmp}/vitype-dmg.XXXXXX")"
cleanup() {
  rm -rf "${STAGING_DIR}" 2>/dev/null || true
}
trap cleanup EXIT

/usr/bin/ditto "${APP_PATH}" "${STAGING_DIR}/${APP_NAME}.app"
ln -s "/Applications" "${STAGING_DIR}/Applications"

DMG_BASENAME="${APP_NAME}-${VERSION}(${BUILD}).dmg"
DMG_PATH="${OUT_DIR}/${DMG_BASENAME}"
rm -f "${DMG_PATH}"

hdiutil create \
  -volname "${VOLNAME}" \
  -srcfolder "${STAGING_DIR}" \
  -format UDZO \
  -imagekey zlib-level=9 \
  -ov \
  "${DMG_PATH}"

echo "Created DMG: ${DMG_PATH}"

# Generate EdDSA signature for Sparkle updates
echo ""
echo "=== Sparkle EdDSA Signing ==="

SPARKLE_SIGN=""
# Try to find sign_update in DerivedData (from SPM build)
if [ -d "${DERIVED_DATA}/SourcePackages/artifacts/sparkle/Sparkle/bin" ]; then
    SPARKLE_SIGN="${DERIVED_DATA}/SourcePackages/artifacts/sparkle/Sparkle/bin/sign_update"
fi

# Fallback: search in common DerivedData locations
if [ ! -x "${SPARKLE_SIGN}" ]; then
    SPARKLE_SIGN=$(find ~/Library/Developer/Xcode/DerivedData -name "sign_update" -type f 2>/dev/null | head -1)
fi

if [ -x "${SPARKLE_SIGN}" ]; then
    echo "Using sign_update: ${SPARKLE_SIGN}"
    EDDSA_OUTPUT=$("${SPARKLE_SIGN}" "${DMG_PATH}" 2>&1 || true)
    if [ -n "${EDDSA_OUTPUT}" ]; then
        echo ""
        echo "EdDSA Signature Info:"
        echo "${EDDSA_OUTPUT}"
        echo ""
        # Extract just the signature value for appcast.xml
        EDDSA_SIG=$(echo "${EDDSA_OUTPUT}" | grep -o 'sparkle:edSignature="[^"]*"' | sed 's/sparkle:edSignature="//;s/"$//' || true)
        if [ -n "${EDDSA_SIG}" ]; then
            echo "sparkle:edSignature value: ${EDDSA_SIG}"
        fi
        echo ""
        echo "DMG file size (bytes) for appcast.xml length attribute:"
        stat -f%z "${DMG_PATH}"
        # Save signature info to file for reference
        echo "${EDDSA_OUTPUT}" > "${DMG_PATH}.eddsa"
        echo ""
        echo "Signature info saved to: ${DMG_PATH}.eddsa"
    else
        echo "warning: Failed to generate EdDSA signature. Is the private key in Keychain?"
        echo "         Run 'generate_keys' from Sparkle to create keys if needed."
    fi
else
    echo "note: Sparkle sign_update tool not found."
    echo "      Build the project in Xcode first to download Sparkle via SPM."
    echo "      Then re-run this script to generate the EdDSA signature."
fi

