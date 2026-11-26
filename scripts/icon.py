"""Prizma app icon: one keycap lit from below by a spectrum glow, a small prism mark on it, on the
app's dark ink tile. Drawn at 4x and downsampled for clean edges."""
import sys
from PIL import Image, ImageDraw, ImageFilter

S = 1024 * 4
out = sys.argv[1]
SPECTRUM = [(255, 61, 110), (255, 157, 46), (255, 225, 77), (77, 255, 145), (61, 214, 255), (107, 92, 255), (210, 77, 255)]


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def spectrum_at(u):
    u = min(max(u, 0.0), 1.0) * (len(SPECTRUM) - 1)
    i = min(int(u), len(SPECTRUM) - 2)
    return lerp(SPECTRUM[i], SPECTRUM[i + 1], u - i)


def gradient(w, h):
    g = Image.new('RGBA', (w, h))
    px = g.load()
    for x in range(w):
        c = spectrum_at(x / (w - 1)) + (255,)
        for y in range(h):
            px[x, y] = c
    return g


img = Image.new('RGBA', (S, S), (0, 0, 0, 0))
pad = int(S * 0.04)
tile = [pad, pad, S - pad, S - pad]
mask = Image.new('L', (S, S), 0)
ImageDraw.Draw(mask).rounded_rectangle(tile, radius=int(S * 0.22), fill=255)
ImageDraw.Draw(img).rounded_rectangle(tile, radius=int(S * 0.22), fill=(16, 13, 28, 255))

# keycap footprint
kx0, ky0, kx1, ky1 = int(S * 0.24), int(S * 0.27), int(S * 0.76), int(S * 0.77)

# spectrum underglow: the footprint filled with the spectrum, blurred wide
glow = Image.new('RGBA', (S, S), (0, 0, 0, 0))
gw, gh = kx1 - kx0 + int(S * 0.06), ky1 - ky0 + int(S * 0.06)
g = gradient(512, 8).resize((gw, gh))
gm = Image.new('L', (gw, gh), 0)
ImageDraw.Draw(gm).rounded_rectangle([0, 0, gw - 1, gh - 1], radius=int(S * 0.08), fill=255)
glow.paste(g, (kx0 - int(S * 0.03), ky0 + int(S * 0.02)), gm)
halo = glow.filter(ImageFilter.GaussianBlur(S * 0.07))
img.alpha_composite(Image.composite(halo, Image.new('RGBA', (S, S)), mask))
img.alpha_composite(Image.composite(halo, Image.new('RGBA', (S, S)), mask))
rim = glow.filter(ImageFilter.GaussianBlur(S * 0.018))
img.alpha_composite(Image.composite(rim, Image.new('RGBA', (S, S)), mask))

# keycap: dark skirt, then a lighter top face inset and lifted
cap = Image.new('RGBA', (S, S), (0, 0, 0, 0))
cd = ImageDraw.Draw(cap)
cd.rounded_rectangle([kx0, ky0, kx1, ky1], radius=int(S * 0.075), fill=(30, 26, 52, 255))
tx0, ty0, tx1, ty1 = kx0 + int(S * 0.05), ky0 + int(S * 0.025), kx1 - int(S * 0.05), ky1 - int(S * 0.085)
top = Image.new('RGBA', (tx1 - tx0, ty1 - ty0))
tp = top.load()
for y in range(ty1 - ty0):
    c = lerp((62, 55, 98), (42, 37, 70), y / (ty1 - ty0 - 1)) + (255,)
    for x in range(tx1 - tx0):
        tp[x, y] = c
tm = Image.new('L', top.size, 0)
ImageDraw.Draw(tm).rounded_rectangle([0, 0, top.size[0] - 1, top.size[1] - 1], radius=int(S * 0.055), fill=255)
cap.paste(top, (tx0, ty0), tm)
img.alpha_composite(cap)

# prism mark on the cap: a plain triangle, centred
mark = Image.new('RGBA', (S, S), (0, 0, 0, 0))
md = ImageDraw.Draw(mark)
mcx, mcy = (tx0 + tx1) / 2, (ty0 + ty1) / 2 + S * 0.012
h = S * 0.2
tri = [(mcx, mcy - h * 0.6), (mcx + h * 0.6, mcy + h * 0.4), (mcx - h * 0.6, mcy + h * 0.4)]
md.line(tri + [tri[0]], fill=(236, 232, 247, 255), width=int(S * 0.02), joint='curve')
img.alpha_composite(mark)

img = img.resize((1024, 1024), Image.LANCZOS)
img.save(out)
print('saved', out)
