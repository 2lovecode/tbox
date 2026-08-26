"""App icon generator: blue disc + white glyph, upscaled from a small source.

Source of truth: `scripts/icon-source.png` (69x71 PNG, white background,
solid indigo-blue disc, white `>_`-style glyph inside the disc).

The source is tiny, so a naive 15x upscale would look soft. Pipeline:

1. Upscale to 1024 with LANCZOS (flat colors upscale well).
2. Mild unsharp mask to keep the glyph crisp.
3. Detect the blue disc (bounding box of blue pixels) and rebuild the
   disc edge as a supersampled antialiased circular alpha mask — the
   outline comes out vector-crisp, and everything outside the disc
   (the white backdrop) becomes transparent.
4. Write `src-tauri/icons/icon.png` (1024x1024, RGBA) — the input for
   `pnpm tauri icon`, which regenerates all platform sizes.

Run after changing the source:
    python3 scripts/make_icon.py && pnpm tauri icon
"""

from PIL import Image, ImageDraw, ImageFilter
import os
import sys

SIZE = 1024
HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "icon-source.png")
OUT = os.path.join(HERE, "..", "src-tauri", "icons", "icon.png")


def blue(p):
    """Source palette: indigo-blue disc on white."""
    r, g, b = p[:3]
    return b > 130 and b > r + 50


def main():
    src = Image.open(SRC).convert("RGB")
    sw, sh = src.size

    # 1. Upscale preserving aspect (fit height), center on square canvas.
    scale = SIZE / sh
    uw = round(sw * scale)
    up = src.resize((uw, SIZE), Image.LANCZOS)
    canvas = Image.new("RGB", (SIZE, SIZE), (255, 255, 255))
    canvas.paste(up, ((SIZE - uw) // 2, 0))

    # 2. Mild sharpen after the big upscale.
    canvas = canvas.filter(
        ImageFilter.UnsharpMask(radius=6, percent=80, threshold=2)
    )

    # 3. Detect the disc from the ORIGINAL pixels (cheap and exact):
    #    bounding box of blue pixels -> center + radius.
    w, h = src.size
    xs, ys = [], []
    for y in range(h):
        for x in range(w):
            if blue(src.getpixel((x, y))):
                xs.append(x)
                ys.append(y)
    if not xs:
        print("no blue disc found in source", file=sys.stderr)
        sys.exit(1)
    cx = (min(xs) + max(xs)) / 2 / sw      # 0..1
    cy = (min(ys) + max(ys)) / 2 / sh
    rr = (max(xs) - min(xs)) / 2 / sw      # relative radius

    # Map onto the upscaled canvas geometry.
    ox = (SIZE - uw) // 2
    CCX = ox + cx * uw
    CCY = cy * SIZE
    RAD = rr * uw
    # Grow very slightly (1px) so the mask never eats into the disc edge.
    RAD += 1

    # 4. Supersampled circular alpha mask (4x, then downsample = antialias).
    ss = 4
    mask = Image.new("L", (SIZE * ss, SIZE * ss), 0)
    ImageDraw.Draw(mask).ellipse(
        (CCX * ss - RAD * ss, CCY * ss - RAD * ss,
         CCX * ss + RAD * ss, CCY * ss + RAD * ss),
        fill=255,
    )
    mask = mask.resize((SIZE, SIZE), Image.LANCZOS)

    icon = canvas.convert("RGBA")
    icon.putalpha(mask)
    icon.save(OUT, "PNG", optimize=True)
    print(f"wrote {OUT} {icon.size} disc=({CCX:.0f},{CCY:.0f}) r={RAD:.0f}")


if __name__ == "__main__":
    main()
