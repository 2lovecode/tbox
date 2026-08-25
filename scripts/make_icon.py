"""Final source icon: monogram logomark.

Concept: a heavy geometric \"T\" (for tbox) set against the slate
backdrop, with a glowing pip in the right end of the top bar and a
small \">_\" terminal glyph underneath as a developer-tool cue.
"""

from PIL import Image, ImageDraw

SIZE = 1024
SS = 4
S = SIZE * SS

# Palette
BG_TOP = (15, 23, 42, 255)       # slate-900
BG_BOT = (2, 6, 23, 255)         # slate-950
T_MAIN = (56, 189, 248, 255)     # sky-400
T_HI = (125, 211, 252, 255)      # sky-300
T_LO = (14, 116, 144, 255)       # sky-700
PIP = (224, 242, 254, 255)       # near-white
TERM = (148, 163, 184, 255)      # slate-400


def lerp(a, b, t):
    return tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(3))


img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Rounded backdrop with vertical gradient.
r = 192 * SS
corner_mask = Image.new("L", (S, S), 0)
ImageDraw.Draw(corner_mask).rounded_rectangle((0, 0, S, S), radius=r, fill=255)
grad = Image.new("RGBA", (S, S), (0, 0, 0, 0))
gd = ImageDraw.Draw(grad)
for i in range(64):
    t = (i + 0.5) / 64
    color = lerp(BG_TOP, BG_BOT, t)
    y0 = int(i * S / 64)
    y1 = int((i + 1) * S / 64)
    gd.rectangle((0, y0, S, y1), fill=color)
img.paste(grad, (0, 0), corner_mask)

# Cyan radial glow centered behind the T.
glow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
gg = ImageDraw.Draw(glow)
cx, cy = S // 2, int(S * 0.47)
for i in range(40, 0, -1):
    a = int(70 * (1 - i / 40) ** 2)
    rr = int(S * 0.45 * i / 40)
    gg.ellipse((cx - rr, cy - rr, cx + rr, cy + rr), fill=(56, 189, 248, a))
img = Image.alpha_composite(img, glow)
draw = ImageDraw.Draw(img)

# The T:
#  vertical stem: a thick rounded rectangle, slightly tapered toward the bottom
#  for a \"sturdy\" look. Width and height picked so the T reads at any size.
stem_w = int(S * 0.18)
stem_top = int(S * 0.30)
stem_bot = int(S * 0.80)
stem_x = (S - stem_w) // 2
draw.rounded_rectangle(
    (stem_x, stem_top, stem_x + stem_w, stem_bot),
    radius=int(stem_w * 0.20),
    fill=T_MAIN,
)
# Top bar of the T: wider, with a tiny chamfer at its outer ends.
bar_h = int(S * 0.18)
bar_x = int(S * 0.16)
bar_w = S - 2 * bar_x
draw.rounded_rectangle(
    (bar_x, int(S * 0.22), bar_x + bar_w, int(S * 0.22) + bar_h),
    radius=int(bar_h * 0.25),
    fill=T_HI,
)
# Subtle inner highlight along the top edge of the bar (sky-200).
hl = Image.new("RGBA", (S, S), (0, 0, 0, 0))
hd = ImageDraw.Draw(hl)
hd.rounded_rectangle(
    (bar_x + int(S * 0.01), int(S * 0.225), bar_x + bar_w - int(S * 0.01), int(S * 0.245)),
    radius=int(bar_h * 0.20),
    fill=(224, 242, 254, 120),
)
img.alpha_composite(hl)
draw = ImageDraw.Draw(img)

# Stem-bar overlap: an inner darker band along the bottom of the bar to give
# the join a tiny shadow (so the stem doesn't look stuck ON TOP of the bar).
shadow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
sd = ImageDraw.Draw(shadow)
sd.rounded_rectangle(
    (bar_x + int(S * 0.012), int(S * 0.22) + bar_h - int(S * 0.022),
     bar_x + bar_w - int(S * 0.012), int(S * 0.22) + bar_h),
    radius=int(bar_h * 0.10),
    fill=(7, 89, 133, 160),
)
img.alpha_composite(shadow)
draw = ImageDraw.Draw(img)

# Stem face: redraw on top of the inner shadow strip so the join is clean.
draw.rounded_rectangle(
    (stem_x, stem_top, stem_x + stem_w, stem_bot),
    radius=int(stem_w * 0.20),
    fill=T_MAIN,
)
# Stem inner highlight strip (vertical).
hl2 = Image.new("RGBA", (S, S), (0, 0, 0, 0))
hd2 = ImageDraw.Draw(hl2)
hd2.rounded_rectangle(
    (stem_x + int(stem_w * 0.12), stem_top + int(stem_w * 0.05),
     stem_x + int(stem_w * 0.32), stem_bot - int(stem_w * 0.10)),
    radius=int(stem_w * 0.10),
    fill=(186, 230, 253, 140),
)
img.alpha_composite(hl2)
draw = ImageDraw.Draw(img)

# Pip: a bright dot with halo, sitting in the right end of the top bar
# (the \"dot of the i\"-equivalent for tbox). Place it INSIDE the bar.
pip_cx = int(S * 0.74)
pip_cy = int(S * 0.30)
pip_r = int(S * 0.050)
# Halo
for i in range(10, 0, -1):
    a = int(90 * (1 - i / 10))
    rr = pip_r + i * 6
    draw.ellipse(
        (pip_cx - rr, pip_cy - rr, pip_cx + rr, pip_cy + rr),
        fill=(125, 211, 252, a),
    )
draw.ellipse(
    (pip_cx - pip_r, pip_cy - pip_r, pip_cx + pip_r, pip_cy + pip_r),
    fill=PIP,
)

# Small \">_ \" terminal glyph tucked under the stem. Three rounded
# rectangles: a chevron, an underscore, and a cursor block.
glyph_y = int(S * 0.86)
glyph_h = int(S * 0.024)


def stick(canvas, x, y, w, h, color, angle=0):
    layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ImageDraw.Draw(layer).rounded_rectangle(
        (x, y, x + w, y + h),
        radius=h // 2,
        fill=color,
    )
    if angle:
        layer = layer.rotate(angle, resample=Image.BICUBIC, center=(x + w // 2, y + h // 2))
    canvas.alpha_composite(layer)


# Cursor block (right).
stick(
    img,
    x=int(S * 0.60),
    y=glyph_y,
    w=glyph_h,
    h=glyph_h,
    color=TERM,
)

# Underscore (middle).
stick(
    img,
    x=int(S * 0.48),
    y=glyph_y + int(S * 0.013),
    w=int(S * 0.10),
    h=int(S * 0.014),
    color=TERM,
)

# \">\" caret (left): a single angled slash reads more clearly than two
# paired lines at small render sizes.
stick(
    img,
    x=int(S * 0.38),
    y=glyph_y,
    w=int(S * 0.080),
    h=glyph_h,
    color=TERM,
    angle=-30,
)

# Downsample.
out = img.resize((SIZE, SIZE), Image.LANCZOS)
out.save(
    "/Users/lh/Documents/dev/projects/hobby/rust/tbox/src-tauri/icons/icon.png",
    "PNG",
    optimize=True,
)
print("wrote icon.png", out.size)
