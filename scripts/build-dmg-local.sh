#!/usr/bin/env bash
set -euo pipefail

# 以專案固定流程建置 DMG，避免手動流程不一致。
pnpm tauri:build:dmg

APP_PATH="src-tauri/target/release/bundle/macos/Voxlore.app"
DMG_DIR="src-tauri/target/release/bundle/dmg"

echo ""
echo "=== 產物檢查 ==="
if [[ -d "$APP_PATH" ]]; then
  echo "App: $APP_PATH"
  /usr/bin/codesign -dvv "$APP_PATH" 2>&1 | sed -n '1,20p'
  /usr/bin/plutil -extract CFBundleIdentifier raw "$APP_PATH/Contents/Info.plist" || true
else
  echo "找不到 app 產物：$APP_PATH"
fi

if [[ -d "$DMG_DIR" ]]; then
  echo "DMG 目錄：$DMG_DIR"
  ls -lah "$DMG_DIR"
else
  echo "找不到 DMG 目錄：$DMG_DIR"
fi
