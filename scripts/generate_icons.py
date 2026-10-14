"""Generate app icons (PNG/ICNS) from src-tauri/icons/source/layout.png (placeholders when absent)
and the menu-bar icon.

The brand artwork lives in icons/source/layout.png; the formats and sizes match
tauri.conf.json so `cargo build` works from a fresh clone. The menu-bar icon is the app icon
at 36 px (tray-app@2x.png) in full colour. It is not a macOS *template* image, so the artwork
keeps its colours instead of becoming a flat silhouette.
"""
import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "src-tauri" / "icons"


def png_chunk(tag: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)


def write_png_rgba(path: Path, size: int, pixel):
    rows = []
    for y in range(size):
        row = bytearray([0])
        for x in range(size):
            row += bytes(pixel(x, y))
        rows.append(bytes(row))
    raw = b"".join(rows)
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", ihdr)
                     + png_chunk(b"IDAT", zlib.compress(raw, 9)) + png_chunk(b"IEND", b""))


def app_icon_pixel(size: int):
    """Rounded dark tile with two light panes (left/right halves)."""
    m = max(1, size // 16)
    inner = size - 2 * m
    pad = max(1, size // 8)
    gap = max(1, size // 32)

    def pixel(x, y):
        if x < m or y < m or x >= size - m or y >= size - m:
            return (0, 0, 0, 0)
        t = (x + y) / (2 * size)
        bg = (int(24 + 30 * t), int(28 + 90 * t), int(40 + 140 * t), 255)
        px, py = x - m, y - m
        if pad <= px < inner - pad and pad <= py < inner - pad:
            mid = inner // 2
            if abs(px - mid) <= gap:
                return bg
            return (235, 240, 250, 255)
        return bg
    return pixel


def write_icns(path: Path):
    def chunk(tag: bytes, data: bytes) -> bytes:
        return tag + struct.pack(">I", len(data) + 8) + data
    body = (chunk(b"ic07", (OUT / "128x128.png").read_bytes())
            + chunk(b"ic08", (OUT / "128x128@2x.png").read_bytes())
            + chunk(b"ic09", (OUT / "icon.png").read_bytes()))
    path.write_bytes(b"icns" + struct.pack(">I", len(body) + 8) + body)


SOURCE = OUT / "source" / "layout.png"  # 실제 아이콘 원본(1254×1254). 있으면 sips로 리사이즈한다.


def resize_from_source():
    """브랜드 원본에서 앱 아이콘 PNG/ICNS를 만든다(macOS sips/iconutil 사용)."""
    import shutil
    import subprocess
    import tempfile

    # tauri-codegen은 RGBA PNG만 받는다 — sips는 원본 색 공간(RGB)을 유지하므로 AppKit으로 다시 인코딩한다.
    # .icns/PNG(구형 경로)는 모서리를 둥글게 잘라 둔다 — macOS 26은 네 귀가 불투명한 정사각형을
    # "구형 아이콘"으로 보고 회색 타일 위에 작게 그린다. Icon Composer 레이어는 1024px 정사각형
    # 원본 그대로(시스템이 마스크를 씌운다).
    converter = Path(__file__).resolve().parent / "png_rgba.swift"
    for size, name in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png"), (512, "icon.png")]:
        subprocess.run(["swift", str(converter), str(SOURCE), str(OUT / name), str(size), "--squircle"], check=True)
    # 메뉴 막대 아이콘: 같은 원본을 36px(18pt @2x)로. 템플릿이 아니라 컬러 그대로 쓴다.
    subprocess.run(["swift", str(converter), str(SOURCE), str(OUT / "tray-app@2x.png"), "36", "--squircle"], check=True)
    with tempfile.TemporaryDirectory() as tmp:
        iconset = Path(tmp) / "app.iconset"
        iconset.mkdir()
        for s in (16, 32, 128, 256, 512):
            subprocess.run(["swift", str(converter), str(SOURCE), str(iconset / f"icon_{s}x{s}.png"), str(s), "--squircle"], check=True)
            subprocess.run(["swift", str(converter), str(SOURCE), str(iconset / f"icon_{s}x{s}@2x.png"), str(s * 2), "--squircle"], check=True)
        subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(OUT / "icon.icns")], check=True)
    composer = OUT / "AppIcon.icon" / "Assets" / "layout.png"
    if composer.parent.exists():
        subprocess.run(["swift", str(converter), str(SOURCE), str(composer), "1024"], check=True)
    shutil.rmtree(OUT / "app.iconset", ignore_errors=True)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    if SOURCE.exists():
        resize_from_source()
    else:
        for size, name in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png"), (512, "icon.png")]:
            write_png_rgba(OUT / name, size, app_icon_pixel(size))
        write_icns(OUT / "icon.icns")
        write_png_rgba(OUT / "tray-app@2x.png", 36, app_icon_pixel(36))
    print(f"wrote icons to {OUT}")


if __name__ == "__main__":
    main()
