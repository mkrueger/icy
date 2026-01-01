#!/bin/bash
# Build script for creating a macOS .app bundle for demo_app
# This is required for testing URL handling (demoapp://...)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
APP_NAME="DemoApp"
BUNDLE_ID="com.icy_ui.demo"
URL_SCHEME="demoapp"

# Build the app
echo "🔨 Building demo_app..."
cd "$PROJECT_ROOT"
cargo build --release -p demo_app

# Create bundle structure
BUNDLE_DIR="$SCRIPT_DIR/$APP_NAME.app"
echo "📦 Creating bundle at $BUNDLE_DIR..."

rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

# Copy executable
cp "$PROJECT_ROOT/target/release/demo_app" "$BUNDLE_DIR/Contents/MacOS/"

# Create Info.plist
cat > "$BUNDLE_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>demo_app</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>Demo App</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>$BUNDLE_ID.url</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>$URL_SCHEME</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create PkgInfo
echo -n "APPL????" > "$BUNDLE_DIR/Contents/PkgInfo"

echo "✅ Bundle created: $BUNDLE_DIR"
echo ""
echo "📝 To test URL handling:"
echo "   1. Start the app:  open $BUNDLE_DIR"
echo "   2. Navigate to the 'Event Log' page in the sidebar"
echo "   3. In another terminal, run:"
echo "      open \"$URL_SCHEME://hello/world\""
echo "      open \"$URL_SCHEME://test?param=value\""
echo ""
echo "🚀 Starting the app now..."
open "$BUNDLE_DIR"
