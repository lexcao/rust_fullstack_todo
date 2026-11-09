#!/usr/bin/env python3
"""用 Pillow 生成静态社交分享图 og.png（蟹记 Crabnote，Logo A 蟹钳）。

字体：英文 Bricolage Grotesque（仓库 assets/，OFL），中文 Noto Sans CJK（系统字体）。
运行：python3 frontend/scripts/generate_og.py
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

WIDTH, HEIGHT = 1200, 630
S = 2  # 先按 2 倍画再缩小，几何图形边缘抗锯齿
ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'og.png'
DISPLAY_FONT = ROOT / 'assets' / 'bricolage-latin.woff2'
CJK_BOLD = Path('/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc')
CJK_REGULAR = Path('/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc')
CJK_SC_INDEX = 2  # .ttc 里简体中文（SC）的序号

BG = '#f4efe7'
INK = '#1a1612'
INK_2 = '#5a5148'
RUST = '#b7472a'
LINE = '#e4dacd'


def display_font(size, weight=700):
    font = ImageFont.truetype(str(DISPLAY_FONT), size)
    try:
        font.set_variation_by_axes([96, weight])  # opsz, wght
    except (OSError, ValueError):
        pass
    return font


def cjk_font(size, bold=True):
    return ImageFont.truetype(str(CJK_BOLD if bold else CJK_REGULAR), size, index=CJK_SC_INDEX)


def draw_claw(draw, cx, cy, r):
    """Logo A：缺口圆（右上方缺 60°）+ 钳口圆点，几何与 logo/a-crabnote-claw.svg 一致。"""
    draw.pieslice((cx - r, cy - r, cx + r, cy + r), start=-30, end=270, fill=RUST)
    k = r / 28
    dx, dy, dr = 14 * k, -22 * k, 6 * k
    draw.ellipse((cx + dx - dr, cy + dy - dr, cx + dx + dr, cy + dy + dr), fill=RUST)


def main():
    image = Image.new('RGB', (WIDTH * S, HEIGHT * S), BG)
    draw = ImageDraw.Draw(image)

    # 点阵底纹，和站点背景一致，只放在右上留白区，不压字
    for x in range(36, WIDTH, 24):
        for y in range(36, HEIGHT, 24):
            if x > 760 and y < 300:
                draw.ellipse(((x - 1) * S, (y - 1) * S, (x + 1) * S, (y + 1) * S), fill='#ddd3c6')

    draw_claw(draw, 158 * S, 250 * S, 78 * S)
    draw.text((270 * S, 312 * S), 'Crabnote', font=display_font(132 * S), fill=INK, anchor='ls')
    draw.text((276 * S, 392 * S), '蟹记 · Rust 全栈待办示例', font=cjk_font(44 * S), fill=RUST, anchor='ls')
    draw.text((80 * S, 470 * S), 'A full-stack Rust to-do demo — Yew (WebAssembly) + actix-web',
              font=display_font(32 * S, 500), fill=INK_2, anchor='ls')
    draw.line((80 * S, 520 * S, 1120 * S, 520 * S), fill=LINE, width=2 * S)
    draw.text((80 * S, 568 * S), 'todos.lexcao.io', font=display_font(28 * S, 600), fill=RUST, anchor='ls')
    image = image.resize((WIDTH, HEIGHT), Image.LANCZOS)
    image.save(OUTPUT, format='PNG', optimize=True)
    print(f'{OUTPUT} ({WIDTH}x{HEIGHT})')


if __name__ == '__main__':
    main()
