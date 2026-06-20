#!/bin/bash

set -e

cd "$(dirname "$0")"

# ─── Code signing & notarization ─────────────────────────────────────────────
# Reads credentials from a .env file at the repo root (gitignored).
# The app-specific password is stored in Keychain — store it once with:
#
#   security add-generic-password \
#     -a "you@example.com" \
#     -s "permute-notarization" \
#     -w "xxxx-xxxx-xxxx-xxxx"
#
# .env format:
#   APPLE_ID=you@example.com
#   APPLE_TEAM_ID=YOURTEAMID
#   APPLE_SIGNING_IDENTITY=Developer ID Application: Your Name (TEAMID)

APPLE_ID=""
APPLE_TEAM_ID=""
APPLE_SIGNING_IDENTITY=""

if [ -f "$(dirname "$0")/.env" ]; then
    while IFS='=' read -r key value; do
        [[ -z "$key" || "$key" == \#* ]] && continue
        case "$key" in
            APPLE_ID)               APPLE_ID="$value" ;;
            APPLE_TEAM_ID)          APPLE_TEAM_ID="$value" ;;
            APPLE_SIGNING_IDENTITY) APPLE_SIGNING_IDENTITY="$value" ;;
        esac
    done < "$(dirname "$0")/.env"
fi

if [ -n "$APPLE_ID" ] && [ -n "$APPLE_TEAM_ID" ] && [ -n "$APPLE_SIGNING_IDENTITY" ]; then
    APPLE_PASSWORD=$(security find-generic-password -a "$APPLE_ID" -s "permute-notarization" -w 2>/dev/null || true)
    if [ -z "$APPLE_PASSWORD" ]; then
        echo "Warning: no Keychain entry found for permute-notarization — skipping notarization."
        APPLE_ID=""
    else
        # Pass signing identity to Tauri but NOT the notarization vars —
        # we notarize manually after fixing the framework signatures.
        export APPLE_SIGNING_IDENTITY
        echo "Code signing enabled: $APPLE_SIGNING_IDENTITY"
    fi
else
    echo "Signing variables not set — building without code signing."
fi

# ─── Build ────────────────────────────────────────────────────────────────────

if [ ! -d "permute-tauri/node_modules" ]; then
    echo "Installing frontend dependencies..."
    cd permute-tauri && npm install && cd ..
fi

echo "Building Permute (universal macOS binary)..."
cd permute-tauri
npm run build:universal
cd ..

# Tauri outputs to the workspace target dir, not src-tauri/target
APP="target/universal-apple-darwin/release/bundle/macos/Permute.app"
DMG_DIR="target/universal-apple-darwin/release/bundle/dmg"

# ─── Re-sign frameworks with hardened runtime ─────────────────────────────────
# Tauri signs the dylib without --options runtime (flags=0x0 instead of
# flags=0x10000), which fails notarization. Re-sign it correctly here.

if [ -n "$APPLE_SIGNING_IDENTITY" ]; then
    echo "Re-signing libsndfile with hardened runtime..."
    codesign --force \
             --sign "$APPLE_SIGNING_IDENTITY" \
             --timestamp \
             --options runtime \
             "$APP/Contents/Frameworks/libsndfile_universal.dylib"

    echo "Re-signing app bundle..."
    codesign --force \
             --sign "$APPLE_SIGNING_IDENTITY" \
             --timestamp \
             --options runtime \
             --entitlements "permute-tauri/src-tauri/Entitlements.plist" \
             "$APP"

    # Verify before submitting
    echo "Verifying signatures..."
    codesign --verify --deep --strict --verbose "$APP"
fi

# ─── Notarize & staple ────────────────────────────────────────────────────────

if [ -n "$APPLE_SIGNING_IDENTITY" ] && [ -n "$APPLE_ID" ] && [ -n "$APPLE_PASSWORD" ] && [ -n "$APPLE_TEAM_ID" ]; then
    echo "Fixing permissions before DMG creation..."
    chmod -R a+rX,u+w "$APP"

    echo "Creating DMG..."
    mkdir -p "$DMG_DIR"
    DMG="$DMG_DIR/Permute_universal.dmg"
    hdiutil create -volname "Permute" \
        -srcfolder "$APP" \
        -ov -format UDZO \
        "$DMG"

    echo "Submitting DMG for notarization (this may take a few minutes)..."
    xcrun notarytool submit "$DMG" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_PASSWORD" \
        --team-id "$APPLE_TEAM_ID" \
        --wait

    echo "Stapling notarization ticket..."
    xcrun stapler staple "$DMG"
    echo "DMG: $DMG"
fi

echo ""
echo "Build complete. Output:"
echo "  $APP"
[ -f "$DMG_DIR/Permute_universal.dmg" ] && echo "  $DMG_DIR/Permute_universal.dmg"
