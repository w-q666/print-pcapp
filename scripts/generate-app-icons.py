"""从 src-tauri/icons/tray-icon.png 生成 Tauri bundle 所需图标（与托盘视觉统一）。"""
from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src-tauri" / "icons" / "tray-icon.png"
OUT_DIR = ROOT / "src-tauri" / "icons"


def main() -> None:
    if not SRC.is_file():
        print(f"缺少源图: {SRC}", file=sys.stderr)
        sys.exit(1)

    im = Image.open(SRC).convert("RGBA")
    # 纯黑背景 → 透明（原型图多为 #000 底，避免任务栏出现黑方块）
    px = im.load()
    w, h = im.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if r < 8 and g < 8 and b < 8 and a > 200:
                px[x, y] = (0, 0, 0, 0)

    def save_scaled(size: int, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        scaled = im.resize((size, size), Image.Resampling.LANCZOS)
        scaled.save(path, format="PNG", optimize=True)

    save_scaled(32, OUT_DIR / "32x32.png")
    save_scaled(128, OUT_DIR / "128x128.png")
    save_scaled(256, OUT_DIR / "128x128@2x.png")

    # 与 bundle 主图一致
    save_scaled(512, OUT_DIR / "icon.png")

    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    ico_images = [
        im.resize(s, Image.Resampling.LANCZOS) for s in ico_sizes
    ]
    ico_images[0].save(
        OUT_DIR / "icon.ico",
        format="ICO",
        sizes=ico_sizes,
        append_images=ico_images[1:],
    )

    # macOS bundle
    im.resize((1024, 1024), Image.Resampling.LANCZOS).save(
        OUT_DIR / "icon.icns", format="ICNS"
    )

    print("已写入:", OUT_DIR)


if __name__ == "__main__":
    main()
