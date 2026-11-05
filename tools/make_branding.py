#!/usr/bin/env python3
"""Builds every logo, icon and brand picture of GxOled from one description of the "Gx" mark.

    python3 tools/make_branding.py

The mark is a pixel "Gx" on an 11 x 7 grid (a 5 x 7 "G", one empty column, a 5 x 5 "x" sitting on the same
baseline) inside a rounded tile. Two variants exist:

* normal       - near-black tile, grey rim, white pixels (the application / "running" icon)
* disconnected - dark-red tile, red rim, salmon pixels (the tray icon while no base station is connected)

Every size is drawn on whole pixels (a cell of size / 16 px, rounded), so the pixels of the mark stay
perfectly sharp in the tray (16/32 px) and in the executable's icon. The files written are listed in `main()`.

Needs Pillow. The fonts are the ones the application embeds (PixelOperator, Inter).
"""
from __future__ import annotations

import io
import struct
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
PIXEL_FONT = ROOT / "gxoled_draw" / "fonts" / "PixelOperator.ttf"
INTER = ROOT / "gxoled_ui" / "fonts"

# ── the mark ─────────────────────────────────────────────────────────────────────────────────────────────

G = ["01110", "10001", "10000", "10111", "10001", "10001", "01110"]
X = ["10001", "01010", "00100", "01010", "10001"]
GRID_W, GRID_H = 11, 7


def cells() -> list[tuple[int, int]]:
    """Lit cells of the 11 x 7 grid as (column, row)."""
    out = [(c, r) for r, row in enumerate(G) for c, ch in enumerate(row) if ch == "1"]
    out += [(6 + c, 2 + r) for r, row in enumerate(X) for c, ch in enumerate(row) if ch == "1"]
    return out


NORMAL = dict(name="normal", fill=(17, 21, 25), ring=(56, 60, 66), glyph=(255, 255, 255), seams=True)
DISCONNECTED = dict(name="disconnected", fill=(128, 25, 21), ring=(170, 43, 43), glyph=(250, 111, 110), seams=False)

ORANGE = (255, 107, 0)  # the accent of the settings window

CORNER = 0.195  # corner radius / tile size
RIM = 0.04  # rim width / tile size


def hexcolor(rgb: tuple[int, int, int]) -> str:
    return "#%02x%02x%02x" % rgb


# ── raster icon ──────────────────────────────────────────────────────────────────────────────────────────


def render(size: int, v: dict) -> Image.Image:
    """The tile with the mark, `size` x `size` px, RGBA, transparent outside the rounded tile."""
    cell = max(1, round(size / 16))  # whole pixels per cell: 16 -> 1, 24 -> 2, 32 -> 2, 48 -> 3, 128 -> 8 ...
    ss = 8 if size <= 64 else 4
    s = size * ss
    radius = round(CORNER * s)
    rim = 0 if size <= 24 else round(RIM * size)

    outer = Image.new("L", (s, s), 0)
    ImageDraw.Draw(outer).rounded_rectangle((0, 0, s - 1, s - 1), radius=radius, fill=255)
    inner = Image.new("L", (s, s), 0)
    ImageDraw.Draw(inner).rounded_rectangle(
        (rim * ss, rim * ss, s - 1 - rim * ss, s - 1 - rim * ss), radius=max(radius - rim * ss, 0), fill=255
    )
    outer = outer.resize((size, size), Image.LANCZOS)
    inner = inner.resize((size, size), Image.LANCZOS)

    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    img.paste(Image.new("RGBA", (size, size), v["ring"] + (255,)), (0, 0), outer)
    img.paste(Image.new("RGBA", (size, size), v["fill"] + (255,)), (0, 0), inner)

    # The 11 x 7 cells, centred (when the margin is odd the extra pixel goes right / below, which makes the mark sit
    # a little above the middle, like in the original drawing).
    x0, y0 = (size - GRID_W * cell) // 2, (size - GRID_H * cell) // 2
    seam = cell // 16 if v["seams"] else 0  # only the big sizes show the hairline between the pixels
    d = ImageDraw.Draw(img)
    for cx, cy in cells():
        px, py = x0 + cx * cell, y0 + cy * cell
        d.rectangle((px, py, px + cell - 1 - seam, py + cell - 1 - seam), fill=v["glyph"] + (255,))
    return img


def png_bytes(img: Image.Image) -> bytes:
    buf = io.BytesIO()
    img.save(buf, "PNG", optimize=True)
    return buf.getvalue()


def ico_bytes(sizes: list[int], v: dict) -> bytes:
    """A Windows .ico with one PNG-compressed image per size (what the build already embeds)."""
    blobs = [(s, png_bytes(render(s, v))) for s in sizes]
    head = struct.pack("<HHH", 0, 1, len(blobs))
    offset = 6 + 16 * len(blobs)
    entries, data = b"", b""
    for s, blob in blobs:
        dim = 0 if s >= 256 else s
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(blob), offset + len(data))
        data += blob
    return head + entries + data


# ── vector master ────────────────────────────────────────────────────────────────────────────────────────


def svg(v: dict, title: str) -> str:
    size = 1024
    cell = size / 16
    rim = RIM * size
    radius = CORNER * size
    x0, y0 = 2.5 * cell, 4.5 * cell
    seam = cell / 16 if v["seams"] else 0
    rects = "\n".join(
        f'    <rect x="{x0 + cx * cell:g}" y="{y0 + cy * cell:g}" width="{cell - seam:g}" height="{cell - seam:g}"/>'
        for cx, cy in cells()
    )
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="512" height="512" role="img" aria-labelledby="t">
  <title id="t">{title}</title>
  <rect width="{size}" height="{size}" rx="{radius:g}" fill="{hexcolor(v['ring'])}"/>
  <rect x="{rim:g}" y="{rim:g}" width="{size - 2 * rim:g}" height="{size - 2 * rim:g}" rx="{radius - rim:g}" fill="{hexcolor(v['fill'])}"/>
  <g fill="{hexcolor(v['glyph'])}" shape-rendering="crispEdges">
{rects}
  </g>
</svg>
"""


# ── lit-pixel text (the OLED look) ───────────────────────────────────────────────────────────────────────


def pixel_mask(text: str, tracking: int = 0) -> Image.Image:
    """`text` in the pixel font at its native size, one image pixel per OLED pixel (no anti-aliasing)."""
    font = ImageFont.truetype(str(PIXEL_FONT), 16)
    widths = [round(font.getlength(ch)) for ch in text]
    w = sum(widths) + tracking * (len(text) - 1) + 2
    im = Image.new("L", (w, 20), 0)
    d = ImageDraw.Draw(im)
    d.fontmode = "1"
    x = 1
    for ch, cw in zip(text, widths):
        d.text((x, 2), ch, font=font, fill=255)
        x += cw + tracking
    box = im.getbbox() or (0, 0, 1, 1)
    return im.crop(box)


def draw_lit(img: Image.Image, mask: Image.Image, x: int, y: int, scale: int, color, seam: int = 1) -> tuple[int, int]:
    """Draw `mask` as lit pixels of `scale` px (with a hairline between them). Returns the size drawn."""
    d = ImageDraw.Draw(img)
    px = mask.load()
    for yy in range(mask.height):
        for xx in range(mask.width):
            if px[xx, yy] > 127:
                d.rectangle(
                    (x + xx * scale, y + yy * scale, x + (xx + 1) * scale - 1 - seam, y + (yy + 1) * scale - 1 - seam),
                    fill=color,
                )
    return mask.width * scale, mask.height * scale


def inter(weight: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(INTER / f"Inter-{weight}.otf"), size)


# ── banner and social preview ────────────────────────────────────────────────────────────────────────────

BG_TOP, BG_BOTTOM = (12, 15, 19), (19, 24, 30)
MUTED = (150, 158, 170)
TEXT = (233, 236, 241)


def backdrop(w: int, h: int, corner: int) -> Image.Image:
    """Dark card: a soft vertical gradient and a faint OLED dot grid fading in from the right."""
    img = Image.new("RGBA", (w, h))
    d = ImageDraw.Draw(img)
    for y in range(h):
        t = y / max(h - 1, 1)
        d.line([(0, y), (w, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(BG_TOP, BG_BOTTOM)) + (255,))
    grid = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    gd = ImageDraw.Draw(grid)
    pitch = 14
    for gy in range(pitch // 2, h, pitch):
        for gx in range(pitch // 2, w, pitch):
            fade = max(0.0, (gx / w - 0.45) / 0.55)
            if fade > 0:
                gd.rectangle((gx, gy, gx + 5, gy + 5), fill=(255, 255, 255, round(26 * fade)))
    img = Image.alpha_composite(img, grid)
    mask = Image.new("L", (w * 2, h * 2), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, w * 2 - 1, h * 2 - 1), radius=corner * 2, fill=255)
    img.putalpha(mask.resize((w, h), Image.LANCZOS))
    return img


def chips(img: Image.Image, x: int, y: int, labels: list[str], font) -> None:
    """A row of rounded outline chips (drawn on their own layer so the translucent fill really blends)."""
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    pad_x, h = 18, 38
    spans = []
    for label in labels:
        tw = round(d.textlength(label, font=font))
        d.rounded_rectangle((x, y, x + tw + 2 * pad_x, y + h), radius=h // 2, fill=(255, 255, 255, 16), outline=(255, 255, 255, 58))
        spans.append((x + pad_x, label))
        x += tw + 2 * pad_x + 10
    img.alpha_composite(layer)
    d = ImageDraw.Draw(img)
    for tx, label in spans:
        d.text((tx, y + h // 2), label, font=font, fill=TEXT + (255,), anchor="lm")


def oled_mock(scale: int) -> Image.Image:
    """An illustration of the 128 x 64 OLED: unlit dots, a lit pixel clock and the brand name."""
    gw, gh, pad = 128, 64, 14
    img = Image.new("RGBA", (gw * scale + 2 * pad, gh * scale + 2 * pad), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((0, 0, img.width - 1, img.height - 1), radius=16, fill=(5, 6, 8, 255), outline=(66, 72, 82, 255), width=2)
    seam = 1 if scale >= 3 else 0
    for gy in range(gh):
        for gx in range(gw):
            x, y = pad + gx * scale, pad + gy * scale
            d.rectangle((x, y, x + scale - 1 - seam, y + scale - 1 - seam), fill=(16, 18, 22, 255))
    lit = (240, 244, 255, 255)

    def put(mask: Image.Image, mag: int, top: int) -> None:
        w = mask.width * mag
        x0 = (gw - w) // 2
        px = mask.load()
        for yy in range(mask.height):
            for xx in range(mask.width):
                if px[xx, yy] > 127:
                    for dy in range(mag):
                        for dx in range(mag):
                            x = pad + (x0 + xx * mag + dx) * scale
                            y = pad + (top + yy * mag + dy) * scale
                            d.rectangle((x, y, x + scale - 1 - seam, y + scale - 1 - seam), fill=lit)

    put(pixel_mask("21:45"), 3, 12)
    put(pixel_mask("GXOLED"), 1, 46)
    return img


TAGLINE = ("Custom graphics, clock, media and GIFs", "for the SteelSeries Arctis Nova OLED.")
FEATURES = ["Clock", "Media", "Animations", "GIF Studio", "Burn-in protection"]


def banner(w: int = 1600, h: int = 420) -> Image.Image:
    img = backdrop(w, h, 28)
    img.alpha_composite(render(256, NORMAL), (72, 72))
    x = 72 + 256 + 60
    tw, th = draw_lit(img, pixel_mask("GxOled"), x, 64, 9, (255, 255, 255, 255))
    sh = draw_lit(img, pixel_mask("ADVANCED", tracking=1), x + 6, 64 + th + 22, 5, ORANGE + (255,))[1]
    d = ImageDraw.Draw(img)
    f = inter("Medium", 26)
    ty = 64 + th + 22 + sh + 30
    for line in TAGLINE:
        d.text((x + 4, ty), line, font=f, fill=TEXT + (255,))
        ty += 38
    chips(img, x + 4, h - 92, FEATURES, inter("Medium", 19))
    oled = oled_mock(3)
    img.alpha_composite(oled, (w - 72 - oled.width, (h - oled.height) // 2 - 8))
    return img


def social(w: int = 1280, h: int = 640) -> Image.Image:
    img = backdrop(w, h, 0)
    img.alpha_composite(render(272, NORMAL), (96, 92))
    x = 96 + 272 + 56
    tw, th = draw_lit(img, pixel_mask("GxOled"), x, 96, 9, (255, 255, 255, 255))
    sh = draw_lit(img, pixel_mask("ADVANCED", tracking=1), x + 6, 96 + th + 24, 5, ORANGE + (255,))[1]
    d = ImageDraw.Draw(img)
    f = inter("Medium", 25)
    ty = 96 + th + 24 + sh + 34
    for line in TAGLINE:
        d.text((x + 4, ty), line, font=f, fill=TEXT + (255,))
        ty += 38
    oled = oled_mock(2)
    img.alpha_composite(oled, (w - 96 - oled.width, 96))
    chips(img, 96, h - 160, FEATURES, inter("Medium", 21))
    d.text((96, h - 78), "github.com/MehmetNY/GxOled-Steelseries-Oled-Control-Panel", font=inter("Regular", 21), fill=MUTED + (255,))
    return img


def gallery_placeholder(n: int, w: int = 960, h: int = 600) -> Image.Image:
    """An empty gallery slot: the README shows docs/images/gallery/NN.png, so replacing the file fills the slot."""
    img = backdrop(w, h, 0)
    d = ImageDraw.Draw(img)
    # dashed frame
    dash, gap, inset = 14, 10, 22
    for x in range(inset, w - inset, dash + gap):
        for y in (inset, h - inset):
            d.line([(x, y), (min(x + dash, w - inset), y)], fill=(78, 84, 94, 255), width=2)
    for y in range(inset, h - inset, dash + gap):
        for x in (inset, w - inset):
            d.line([(x, y), (x, min(y + dash, h - inset))], fill=(78, 84, 94, 255), width=2)
    num = pixel_mask(f"{n:02d}")
    scale = 14
    nw = num.width * scale
    draw_lit(img, num, (w - nw) // 2, 150, scale, (255, 255, 255, 255))
    f1, f2 = inter("Medium", 30), inter("Regular", 22)
    d.text((w // 2, 360), "Your screenshot goes here", font=f1, fill=TEXT + (255,), anchor="mm")
    d.text((w // 2, 410), f"replace docs/images/gallery/{n:02d}.png", font=f2, fill=MUTED + (255,), anchor="mm")
    return img


def is_placeholder(path: Path) -> bool:
    """True when `path` still holds the generated placeholder of its slot (so regenerating may refresh it)."""
    try:
        n = int(path.stem)
        return path.read_bytes() == png_bytes(gallery_placeholder(n))
    except (ValueError, OSError):
        return False


# ── writing everything ───────────────────────────────────────────────────────────────────────────────────


def write(path: Path, data: bytes | str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(data, str):
        path.write_text(data, encoding="utf-8", newline="\n")
    else:
        path.write_bytes(data)
    print("wrote", path.relative_to(ROOT), f"({path.stat().st_size} bytes)")


def main() -> None:
    assets = ROOT / "gxoled_app" / "assets"
    # Application: exe icon, 128 px icons (Flatpak, documents) and the tray icons (exactly 32 px: 2 px cells).
    write(assets / "gxoled.ico", ico_bytes([16, 20, 24, 32, 40, 48, 64, 128, 256], NORMAL))
    write(assets / "gxoled.png", png_bytes(render(128, NORMAL)))
    write(assets / "tray.png", png_bytes(render(32, NORMAL)))
    write(assets / "tray_error.png", png_bytes(render(32, DISCONNECTED)))
    # Vector masters and the sizes the documents use.
    branding = ROOT / "assets" / "branding"
    write(branding / "gxoled-icon.svg", svg(NORMAL, "GxOled"))
    write(branding / "gxoled-icon-disconnected.svg", svg(DISCONNECTED, "GxOled - no base station connected"))
    logo = ROOT / "docs" / "images" / "logo"
    for s in (512, 256, 128, 64, 32, 16):
        write(logo / f"gxoled-{s}.png", png_bytes(render(s, NORMAL)))
    for s in (256, 32):
        write(logo / f"gxoled-disconnected-{s}.png", png_bytes(render(s, DISCONNECTED)))
    gallery = ROOT / "docs" / "images" / "gallery"
    for n in range(1, 10):
        target = gallery / f"{n:02d}.png"
        # Never overwrite a picture the owner has put in a slot: only empty slots get the placeholder.
        if not target.exists() or target.stat().st_size == 0 or is_placeholder(target):
            write(target, png_bytes(gallery_placeholder(n)))
    write(ROOT / "docs" / "images" / "banner.png", png_bytes(banner()))
    write(ROOT / "docs" / "images" / "social-preview.png", png_bytes(social()))


if __name__ == "__main__":
    main()
