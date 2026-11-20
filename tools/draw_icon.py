"""Fluxxy's PvP Pack icon: 32x32 pixel art drawn by hand on a grid, scaled to 256x256 without smoothing."""
from PIL import Image

N = 32
img = Image.new("RGBA", (N, N), (0, 0, 0, 0))
px = img.load()

def put(x, y, c):
    if 0 <= x < N and 0 <= y < N:
        px[x, y] = c

# Background: a rounded tile, dark blue-grey, lighter toward the top.
for y in range(N):
    for x in range(N):
        corner = min(x, N - 1 - x) + min(y, N - 1 - y)
        if (min(x, N - 1 - x) < 2 and min(y, N - 1 - y) < 2 and corner < 2):
            continue
        t = y / (N - 1)
        base = (int(44 - 18 * t), int(48 - 20 * t), int(66 - 26 * t), 255)
        put(x, y, base)
# Gold rim.
GOLD, GOLD_DARK = (255, 200, 40, 255), (190, 130, 20, 255)
for i in range(2, N - 2):
    put(i, 0, GOLD); put(i, N - 1, GOLD_DARK); put(0, i, GOLD); put(N - 1, i, GOLD_DARK)
for (x, y) in [(1, 1), (N - 2, 1), (1, N - 2), (N - 2, N - 2)]:
    put(x, y, GOLD if y == 1 else GOLD_DARK)

# Low flames along the bottom: three tongues, red edge, orange body, yellow core.
RED, ORANGE, YELLOW, WHITE_HOT = (200, 40, 20, 255), (255, 120, 20, 255), (255, 210, 60, 255), (255, 245, 190, 255)
heights = [3, 5, 8, 6, 4, 6, 9, 7, 5, 4, 6, 8, 10, 8, 6, 4, 5, 7, 9, 7, 5, 3, 4, 6, 8, 6, 4, 3]
for i, h in enumerate(heights):
    x = 2 + i
    for k in range(h):
        y = N - 2 - k
        if k >= h - 1:
            c = RED
        elif k >= h - 3:
            c = ORANGE
        elif k >= 2 and h >= 6:
            c = YELLOW if k < h - 3 else ORANGE
        else:
            c = YELLOW
        if h >= 8 and 2 <= k <= 3:
            c = WHITE_HOT
        put(x, y, c)

# Sword on its own layer, then outlined: diagonal from the pommel (bottom left) to the tip (top right).
layer = {}
def s(x, y, c):
    layer[(x, y)] = c
OUT = (18, 18, 26, 255)
BLADE_L, BLADE, BLADE_D = (240, 244, 252, 255), (196, 206, 222, 255), (128, 138, 160, 255)
for t in range(14):                       # blade: 3 pixels across the diagonal
    x, y = 13 + t, 18 - t
    s(x, y - 1, BLADE_L); s(x, y, BLADE); s(x + 1, y, BLADE); s(x, y + 1, BLADE_D) if t < 13 else None
    s(x + 1, y + 1, BLADE_D) if t < 12 else None
s(27, 3, BLADE_L); s(28, 3, BLADE_L)      # tip
for d in range(-3, 4):                    # guard, across the blade
    s(12 + d, 19 + d, GOLD if d < 1 else GOLD_DARK); s(13 + d, 19 + d, GOLD if d < 0 else GOLD_DARK)
BROWN, BROWN_D = (128, 78, 42, 255), (84, 48, 26, 255)
for t in range(4):                        # grip
    x, y = 11 - t, 21 + t
    s(x, y, BROWN if t % 2 == 0 else BROWN_D); s(x + 1, y, BROWN_D if t % 2 == 0 else BROWN)
for (x, y) in [(6, 25), (7, 25), (6, 26), (7, 26)]:   # pommel
    s(x, y, GOLD if (x, y) == (6, 25) else GOLD_DARK)
for (x, y) in list(layer):                # outline every edge of the sword
    for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
        if (x + dx, y + dy) not in layer:
            put(x + dx, y + dy, OUT)
for (x, y), c in layer.items():
    put(x, y, c)

# Saturation: three hunger shanks top left; the first two wear the gold outline the mod draws.
SHANK = [".MM..", "MMMM.", "MMMm.", ".mmB.", "...BB"]
MEAT, MEAT_D, BONE = (214, 120, 62, 255), (160, 78, 40, 255), (240, 232, 214, 255)
for j in range(3):
    x0, y0 = 3 + j * 6, 4
    cells = {(x0 + c, y0 + r): ch for r, row in enumerate(SHANK) for c, ch in enumerate(row) if ch != "."}
    ring = GOLD if j < 2 else OUT
    for (x, y) in cells:
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            if (x + dx, y + dy) not in cells:
                put(x + dx, y + dy, ring)
    for (x, y), ch in cells.items():
        put(x, y, {"M": MEAT, "m": MEAT_D, "B": BONE}[ch])

img.resize((256, 256), Image.NEAREST).save("icon.png")
