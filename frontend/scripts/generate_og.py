#!/usr/bin/env python3
"""Generate the static social preview with Pillow and local DejaVu fonts."""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


WIDTH, HEIGHT = 1200, 630
FONT_DIR = Path('/usr/share/fonts/truetype/dejavu')
OUTPUT = Path(__file__).resolve().parents[1] / 'og.png'


def main():
    image = Image.new('RGB', (WIDTH, HEIGHT), '#fff8ef')
    draw = ImageDraw.Draw(image)
    title_font = ImageFont.truetype(str(FONT_DIR / 'DejaVuSans-Bold.ttf'), 70)
    subtitle_font = ImageFont.truetype(str(FONT_DIR / 'DejaVuSans.ttf'), 34)
    footer_font = ImageFont.truetype(str(FONT_DIR / 'DejaVuSans.ttf'), 28)

    draw.rectangle((0, 0, 20, HEIGHT), fill='#b7472a')
    draw.rectangle((80, 120, 180, 130), fill='#b7472a')
    draw.text((80, 210), 'Rust Fullstack Todo', font=title_font, fill='#29231f')
    draw.text((80, 320), 'A Rust / Yew + actix-web learning demo',
              font=subtitle_font, fill='#604d43')
    draw.line((80, 480, 1120, 480), fill='#dfcabb', width=2)
    draw.text((80, 520), 'todos.lexcao.io', font=footer_font, fill='#b7472a')
    image.save(OUTPUT, format='PNG', optimize=True)
    print(f'{OUTPUT} ({WIDTH}x{HEIGHT})')


if __name__ == '__main__':
    main()
