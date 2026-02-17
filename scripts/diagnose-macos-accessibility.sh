#!/usr/bin/env bash
set -euo pipefail

APP_PATH="${1:-/Applications/Voxlore.app}"

echo "=== Voxlore macOS 診斷 ==="
echo "App 路徑: $APP_PATH"

if [[ ! -d "$APP_PATH" ]]; then
  echo "找不到 app：$APP_PATH"
  exit 1
fi

echo ""
echo "[1] Bundle ID"
/usr/bin/plutil -extract CFBundleIdentifier raw "$APP_PATH/Contents/Info.plist"

echo ""
echo "[2] Code Sign"
/usr/bin/codesign -dvv "$APP_PATH" 2>&1 | sed -n '1,40p'

echo ""
echo "[3] libvosk Code Sign"
if [[ -f "$APP_PATH/Contents/Frameworks/libvosk.dylib" ]]; then
  /usr/bin/codesign -dvv "$APP_PATH/Contents/Frameworks/libvosk.dylib" 2>&1 | sed -n '1,40p'
else
  echo "找不到 libvosk.dylib"
fi

echo ""
echo "[4] Gatekeeper 評估"
/usr/sbin/spctl --assess --type execute -vv "$APP_PATH" || true

echo ""
echo "[5] 建議下一步"
echo "- 確認 系統設定 > 隱私權與安全性 > 輔助使用 內的 Voxlore 已啟用"
echo "- 若剛重裝，請先完全關閉 App 再重開一次後重試自動輸入"
