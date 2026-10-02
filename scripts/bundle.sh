#!/bin/sh
# Builds target/GnaegiCut.app and target/GnaegiCut.dmg. Needs: cargo, models/ggml-base.bin (sh scripts/get-model.sh).
set -e
cd "$(dirname "$0")/.."
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
[ -f models/ggml-base.bin ] || sh scripts/get-model.sh
cargo build --release
APP=target/GnaegiCut.app
rm -rf "$APP" && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/models"
cp target/release/gnaegicut "$APP/Contents/MacOS/gnaegicut"
cp -R assets "$APP/Contents/Resources/assets"
cp models/ggml-base.bin "$APP/Contents/Resources/models/"
mkdir -p target/icon.iconset
for s in 16 32 128 256 512; do
  sips -z $s $s assets/icon.png --out target/icon.iconset/icon_${s}x${s}.png >/dev/null
  sips -z $((s*2)) $((s*2)) assets/icon.png --out target/icon.iconset/icon_${s}x${s}@2x.png >/dev/null
done
iconutil -c icns target/icon.iconset -o "$APP/Contents/Resources/icon.icns"
cat > "$APP/Contents/Info.plist" <<P
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>GnaegiCut</string>
<key>CFBundleIdentifier</key><string>ch.gnaegi.gnaegicut</string>
<key>CFBundleExecutable</key><string>gnaegicut</string>
<key>CFBundleIconFile</key><string>icon</string>
<key>CFBundleVersion</key><string>$VERSION</string>
<key>CFBundleShortVersionString</key><string>$VERSION</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
P
codesign --force --deep --sign - "$APP"
rm -rf target/dmg && mkdir target/dmg && cp -R "$APP" target/dmg/ && ln -s /Applications target/dmg/Applications
rm -f target/GnaegiCut.dmg
hdiutil create -volname GnaegiCut -srcfolder target/dmg -ov -format UDZO target/GnaegiCut.dmg
echo "Built target/GnaegiCut.dmg"
