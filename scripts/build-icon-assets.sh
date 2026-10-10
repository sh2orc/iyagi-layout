#!/bin/bash
# build-icon-assets.sh — macOS 26(Tahoe)용 아이콘 에셋 컴파일.
#
# Tahoe는 .icns만 있는 앱을 "구형 아이콘" 취급해 회색 둥근 사각형 안에 흐릿하게 그린다.
# Icon Composer 형식(src-tauri/icons/AppIcon.icon)을 actool로 Assets.car로 컴파일해
# 번들 Contents/Resources에 넣고(tauri.conf.json bundle.macOS.files), Info.plist의
# CFBundleIconName=AppIcon이 그것을 가리킨다. Tauri 2.12 자체에는 이 기능이 없다.
# Xcode 26의 actool이 없으면 경고만 하고 넘어간다(.icns 폴백으로 동작).
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
src="$root/src-tauri/icons/AppIcon.icon"
out="$root/src-tauri/icons/build"
mkdir -p "$out"
if ! xcrun --find actool >/dev/null 2>&1; then
  echo "build-icon-assets: actool not found (Xcode 26 needed) — keeping .icns only" >&2
  exit 0
fi
tmp=$(mktemp -d)
if xcrun actool "$src" --compile "$tmp" --app-icon AppIcon --include-all-app-icons \
  --output-partial-info-plist "$tmp/partial.plist" --platform macosx \
  --minimum-deployment-target 13.0 >"$tmp/actool.log" 2>&1 && [ -f "$tmp/Assets.car" ]; then
  cp -f "$tmp/Assets.car" "$out/Assets.car"
  echo "build-icon-assets: wrote $out/Assets.car"
else
  echo "build-icon-assets: actool failed — keeping .icns only" >&2
  tail -5 "$tmp/actool.log" >&2
fi
rm -rf "$tmp"
exit 0
