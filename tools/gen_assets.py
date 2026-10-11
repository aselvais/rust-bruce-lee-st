#!/usr/bin/env python3
"""Pixel-art assets for the Level 1 tribute. Original art, one shared palette."""

import math
import os
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "assets")
os.makedirs(OUT, exist_ok=True)

# (mid, light, shadow)
SKIN = ((214, 156, 112), (250, 206, 164), (158, 96, 68))
GI = ((236, 184, 52), (255, 224, 128), (168, 108, 28))
PANTS = ((32, 28, 44), (62, 56, 78), (16, 14, 24))
SHOE = ((22, 18, 28), (48, 42, 56), (10, 8, 14))
HAIR = ((18, 14, 22), (46, 40, 52), (8, 6, 10))
INK = (12, 10, 16, 255)
RED = ((176, 40, 46), (230, 96, 84), (110, 22, 32))
WOOD = ((148, 96, 52), (196, 144, 88), (96, 60, 32))
NINJA_C = ((34, 32, 46), (78, 76, 98), (16, 14, 24))
YAMO_C = ((52, 138, 70), (118, 196, 112), (26, 84, 46))
STONE = ((108, 98, 102), (164, 152, 140), (62, 56, 64))
STONE_DK = ((62, 56, 66), (96, 88, 96), (36, 32, 40))
GOLD = ((230, 176, 56), (255, 224, 130), (160, 104, 28))
VINE_C = ((36, 110, 150), (96, 176, 206), (18, 64, 96))
LEAF = ((42, 120, 72), (86, 168, 96), (24, 72, 44))


def new(w, h):
    return Image.new("RGBA", (w, h), (0, 0, 0, 0))


def dist_seg(px, py, ax, ay, bx, by):
    abx, aby = bx - ax, by - ay
    l2 = abx * abx + aby * aby
    if l2 < 1e-6:
        dx, dy = px - ax, py - ay
        return math.hypot(dx, dy), ax, ay
    t = max(0.0, min(1.0, ((px - ax) * abx + (py - ay) * aby) / l2))
    cx, cy = ax + abx * t, ay + aby * t
    return math.hypot(px - cx, py - cy), cx, cy


def capsule(px, w, h, ax, ay, bx, by, r, col):
    mid, hi, sh = col
    x0 = int(min(ax, bx) - r - 2)
    x1 = int(max(ax, bx) + r + 2)
    y0 = int(min(ay, by) - r - 2)
    y1 = int(max(ay, by) + r + 2)
    for y in range(max(0, y0), min(h, y1 + 1)):
        for x in range(max(0, x0), min(w, x1 + 1)):
            d, cx, cy = dist_seg(x + 0.5, y + 0.5, ax, ay, bx, by)
            if d <= r:
                nx, ny = (x + 0.5 - cx) / (r + 0.01), (y + 0.5 - cy) / (r + 0.01)
                light = nx * -0.65 + ny * -0.75
                if light > 0.35:
                    rgb = hi
                elif light < -0.25:
                    rgb = sh
                else:
                    rgb = mid
                px[x, y] = (*rgb, 255)


def disc(px, w, h, cx, cy, r, col):
    capsule(px, w, h, cx, cy, cx, cy, r, col)


def outline(im):
    px = im.load()
    w, h = im.size
    add = []
    for y in range(h):
        for x in range(w):
            if px[x, y][3] < 40:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and px[nx, ny][3] < 40:
                    add.append((nx, ny))
                elif not (0 <= nx < w and 0 <= ny < h):
                    pass
    for x, y in add:
        if px[x, y][3] < 40:
            px[x, y] = INK


def rel(anchor, p, scale):
    return (anchor[0] + p[0] * scale, anchor[1] + p[1] * scale)


# Joints are offsets from the feet anchor. Negative y is up. Facing right.
POSES = {
    # Kung-fu guard: lead fist out, rear fist on the ribs, knees soft.
    "idle0": dict(hip=(0, -20), chest=(3, -34), head=(5, -47), e1=(6, -32), h1=(16, -28), e2=(-2, -30), h2=(-8, -26), k1=(5, -11), f1=(8, -4), k2=(-5, -11), f2=(-6, -4)),
    "idle1": dict(hip=(0, -21), chest=(3, -35), head=(5, -48), e1=(7, -33), h1=(17, -30), e2=(-1, -31), h2=(-7, -27), k1=(5, -11), f1=(8, -4), k2=(-5, -11), f2=(-6, -4)),
    "run0": dict(hip=(1, -21), chest=(4, -35), head=(6, -48), e1=(-8, -32), h1=(-14, -24), e2=(8, -30), h2=(14, -22), k1=(12, -12), f1=(16, -4), k2=(-10, -14), f2=(-14, -6)),
    "run1": dict(hip=(0, -19), chest=(3, -33), head=(5, -46), e1=(-4, -28), h1=(-8, -18), e2=(6, -32), h2=(10, -24), k1=(6, -8), f1=(4, -4), k2=(-4, -8), f2=(-2, -4)),
    "run2": dict(hip=(0, -22), chest=(2, -36), head=(4, -49), e1=(8, -34), h1=(14, -26), e2=(-8, -30), h2=(-14, -22), k1=(-8, -18), f1=(-4, -10), k2=(10, -12), f2=(14, -4)),
    "run3": dict(hip=(0, -23), chest=(2, -37), head=(4, -50), e1=(10, -36), h1=(8, -26), e2=(-8, -32), h2=(-6, -22), k1=(-10, -20), f1=(-6, -12), k2=(6, -14), f2=(4, -4)),
    "run4": dict(hip=(-1, -21), chest=(2, -35), head=(4, -48), e1=(8, -30), h1=(14, -22), e2=(-8, -32), h2=(-14, -24), k1=(-12, -12), f1=(-16, -4), k2=(10, -14), f2=(14, -6)),
    "run5": dict(hip=(0, -19), chest=(3, -33), head=(5, -46), e1=(4, -28), h1=(8, -18), e2=(-6, -32), h2=(-10, -24), k1=(-6, -8), f1=(-4, -4), k2=(5, -10), f2=(3, -4)),
    "jump": dict(hip=(0, -26), chest=(2, -40), head=(4, -52), e1=(-8, -44), h1=(-4, -54), e2=(10, -38), h2=(16, -30), k1=(-6, -16), f1=(-2, -8), k2=(8, -16), f2=(6, -8)),
    "fall": dict(hip=(0, -22), chest=(2, -36), head=(5, -48), e1=(-12, -34), h1=(-18, -28), e2=(12, -34), h2=(18, -26), k1=(-5, -14), f1=(-8, -6), k2=(8, -12), f2=(12, -4)),
    "duck": dict(hip=(4, -12), chest=(10, -18), head=(14, -28), e1=(14, -16), h1=(22, -14), e2=(2, -16), h2=(-4, -14), k1=(12, -8), f1=(16, -4), k2=(-2, -8), f2=(2, -4)),
    "climb0": dict(hip=(0, -22), chest=(1, -36), head=(1, -48), e1=(6, -46), h1=(8, -56), e2=(-5, -28), h2=(-6, -18), k1=(4, -12), f1=(5, -4), k2=(-4, -16), f2=(-5, -8)),
    "climb1": dict(hip=(0, -22), chest=(1, -36), head=(1, -48), e1=(-5, -46), h1=(-6, -56), e2=(6, -28), h2=(7, -18), k1=(-4, -12), f1=(-5, -4), k2=(4, -16), f2=(5, -8)),
    # Straight punch: chamber, full extension, recover.
    "punch0": dict(hip=(0, -20), chest=(2, -34), head=(4, -47), e1=(-10, -32), h1=(-16, -30), e2=(2, -30), h2=(8, -26), k1=(4, -11), f1=(7, -4), k2=(-5, -11), f2=(-4, -4)),
    "punch1": dict(hip=(-2, -20), chest=(4, -34), head=(6, -47), e1=(12, -34), h1=(26, -34), e2=(-8, -30), h2=(-14, -28), k1=(8, -11), f1=(12, -4), k2=(-6, -11), f2=(-2, -4)),
    "punch2": dict(hip=(-1, -20), chest=(3, -34), head=(5, -47), e1=(8, -33), h1=(18, -32), e2=(-6, -30), h2=(-10, -26), k1=(6, -11), f1=(9, -4), k2=(-5, -11), f2=(-3, -4)),
    # Chambered knee, standing side kick, flying jump kick, recover.
    "kick0": dict(hip=(-1, -20), chest=(1, -35), head=(3, -48), e1=(-8, -32), h1=(-12, -24), e2=(6, -36), h2=(10, -28), k1=(4, -22), f1=(8, -30), k2=(-4, -12), f2=(-2, -4)),
    "kick1": dict(hip=(-4, -20), chest=(-2, -35), head=(0, -48), e1=(-10, -30), h1=(-14, -22), e2=(4, -38), h2=(8, -32), k1=(10, -22), f1=(24, -20), k2=(-4, -12), f2=(-1, -4)),
    "kick2": dict(hip=(-6, -24), chest=(-16, -22), head=(-24, -24), e1=(-12, -30), h1=(-4, -34), e2=(-20, -16), h2=(-28, -14), k1=(6, -22), f1=(24, -20), k2=(-12, -16), f2=(-18, -10)),
    "kick3": dict(hip=(-2, -20), chest=(1, -34), head=(3, -47), e1=(-6, -32), h1=(-8, -24), e2=(6, -34), h2=(10, -28), k1=(8, -16), f1=(16, -10), k2=(-4, -12), f2=(-2, -4)),
    "hurt": dict(hip=(-4, -18), chest=(-8, -32), head=(-10, -44), e1=(-14, -26), h1=(-18, -20), e2=(2, -30), h2=(8, -24), k1=(-2, -11), f1=(2, -4), k2=(-8, -11), f2=(-10, -4)),
    "down": dict(hip=(-6, -8), chest=(8, -9), head=(20, -11), e1=(10, -14), h1=(16, -12), e2=(0, -12), h2=(4, -8), k1=(-16, -8), f1=(-24, -6), k2=(-12, -6), f2=(-20, -5)),
    "win": dict(hip=(0, -20), chest=(2, -35), head=(3, -48), e1=(2, -48), h1=(4, -60), e2=(-6, -30), h2=(-10, -24), k1=(5, -11), f1=(7, -4), k2=(-5, -11), f2=(-4, -4)),
    # Ninja throw: star in the hand, release, follow-through.
    "throw0": dict(hip=(0, -20), chest=(1, -34), head=(3, -47), e1=(-8, -36), h1=(-14, -40), e2=(4, -30), h2=(8, -24), k1=(4, -11), f1=(6, -4), k2=(-5, -11), f2=(-4, -4)),
    "throw1": dict(hip=(-1, -20), chest=(3, -34), head=(5, -47), e1=(10, -34), h1=(22, -32), e2=(-6, -30), h2=(-10, -26), k1=(6, -11), f1=(9, -4), k2=(-5, -11), f2=(-3, -4)),
    "throw2": dict(hip=(0, -20), chest=(2, -34), head=(4, -47), e1=(6, -30), h1=(12, -24), e2=(-4, -32), h2=(-8, -26), k1=(4, -11), f1=(7, -4), k2=(-5, -11), f2=(-4, -4)),
}

HERO_ORDER = [
    "idle0", "idle1", "run0", "run1", "run2", "run3", "run4", "run5",
    "jump", "fall", "duck", "climb0", "climb1", "punch0", "punch1", "punch2",
    "kick0", "kick1", "kick2", "kick3", "hurt", "down", "win",
]
NINJA_ORDER = [
    "idle0", "run0", "run1", "run2", "run3", "punch0", "punch1", "punch2",
    "hurt", "down", "climb0", "climb1", "throw0", "throw1", "throw2",
]
YAMO_ORDER = ["idle0", "run0", "run1", "run2", "run3", "kick0", "kick1", "kick2", "hurt", "down"]


STEEL = ((168, 178, 190), (236, 242, 248), (78, 86, 98))
BELT = ((16, 14, 20), (48, 44, 58), (8, 6, 12))
SOLE = ((210, 206, 196), (240, 236, 226), (140, 132, 120))


def pix(px, w, h, x, y, rgb):
    x, y = int(x), int(y)
    if 0 <= x < w and 0 <= y < h:
        px[x, y] = (*rgb, 255)


def shoe(px, w, h, x, y, col):
    """Side-view kung-fu shoe. x,y is the ankle."""
    capsule(px, w, h, x - 1, y, x + 5, y + 1, 2.3, col)
    capsule(px, w, h, x - 1, y + 2, x + 6, y + 2, 1.3, SOLE)


def fist(px, w, h, x, y, col, big=False):
    r = 3.1 if big else 2.5
    disc(px, w, h, x, y, r, col)
    pix(px, w, h, x + 1, y - 1, (255, 236, 214))
    pix(px, w, h, x + 2, y, (120, 64, 48))
    pix(px, w, h, x + 1, y + 1, (120, 64, 48))


def hero_head(px, w, h, cx, cy):
    """Stylized profile. Black hair, sideburn, sharp brow. A game fighter, not a portrait."""
    disc(px, w, h, cx - 3, cy - 1, 7.2, HAIR)
    disc(px, w, h, cx - 1, cy - 6, 4.6, HAIR)
    disc(px, w, h, cx - 5, cy + 2, 3.4, HAIR)
    disc(px, w, h, cx + 1, cy + 1, 5.6, SKIN)
    capsule(px, w, h, cx - 2, cy - 2, cx - 1, cy + 5, 2.1, HAIR)
    # Brow, eye, nose, mouth.
    for i in range(4):
        pix(px, w, h, cx + 1 + i, cy - 2, (42, 28, 24))
    pix(px, w, h, cx + 2, cy - 1, (42, 28, 24))
    pix(px, w, h, cx + 3, cy, (250, 246, 236))
    pix(px, w, h, cx + 4, cy, (250, 246, 236))
    pix(px, w, h, cx + 4, cy + 1, (18, 14, 20))
    pix(px, w, h, cx + 5, cy + 1, (214, 156, 112))
    pix(px, w, h, cx + 6, cy + 1, (186, 120, 84))
    pix(px, w, h, cx + 6, cy + 2, (158, 96, 68))
    pix(px, w, h, cx + 3, cy + 4, (150, 78, 70))
    pix(px, w, h, cx + 4, cy + 4, (120, 56, 56))


def ninja_head(px, w, h, cx, cy):
    disc(px, w, h, cx, cy, 6.4, NINJA_C)
    disc(px, w, h, cx - 1, cy - 5, 3.6, NINJA_C)
    # Hood tail and red headband.
    capsule(px, w, h, cx - 4, cy - 1, cx - 10, cy + 2, 1.8, NINJA_C)
    capsule(px, w, h, cx - 5, cy - 4, cx + 4, cy - 4, 1.6, RED)
    capsule(px, w, h, cx - 6, cy - 3, cx - 12, cy + 1, 1.2, RED)
    for i in range(6):
        pix(px, w, h, cx + i - 1, cy, (236, 226, 206))
    pix(px, w, h, cx + 1, cy, (16, 12, 18))
    pix(px, w, h, cx + 2, cy, (16, 12, 18))
    pix(px, w, h, cx + 4, cy, (16, 12, 18))


def yamo_head(px, w, h, cx, cy):
    disc(px, w, h, cx, cy + 1, 10.5, YAMO_C)
    # Chonmage topknot.
    disc(px, w, h, cx - 1, cy - 11, 3.6, HAIR)
    capsule(px, w, h, cx - 1, cy - 8, cx - 1, cy - 13, 2.2, HAIR)
    disc(px, w, h, cx + 2, cy - 14, 2.4, HAIR)
    # Heavy brow, squint, scowl.
    for i in range(7):
        pix(px, w, h, cx + i, cy - 2, (16, 48, 28))
        pix(px, w, h, cx + i, cy - 1, (12, 36, 22))
    pix(px, w, h, cx + 3, cy + 1, (240, 236, 220))
    pix(px, w, h, cx + 4, cy + 1, (240, 236, 220))
    pix(px, w, h, cx + 4, cy + 2, (16, 20, 16))
    pix(px, w, h, cx + 6, cy + 2, (240, 236, 220))
    pix(px, w, h, cx + 7, cy + 1.5, (16, 20, 16))
    pix(px, w, h, cx + 8, cy + 3, (40, 90, 48))
    pix(px, w, h, cx + 9, cy + 4, (30, 70, 40))
    for i in range(4):
        pix(px, w, h, cx + 3 + i, cy + 6, (90, 24, 32))
    pix(px, w, h, cx + 4, cy + 7, (210, 170, 150))


def saber(px, w, h, hx, hy, pose):
    """Curved saber in the lead hand. Held down, raised, or slashing."""
    if pose == "punch0":
        tip = (hx + 4, hy - 18)
    elif pose == "punch1":
        tip = (hx + 20, hy - 2)
    elif pose == "punch2":
        tip = (hx + 12, hy + 10)
    elif pose in ("throw0", "throw1", "throw2"):
        return
    elif pose in ("climb0", "climb1"):
        tip = (hx + 2, hy + 14)
    elif pose.startswith("run"):
        tip = (hx - 4, hy + 16)
    else:
        tip = (hx + 12, hy + 16)
    # Glove, wrapped handle, round guard, then a curved silver blade.
    disc(px, w, h, hx, hy, 2.6, NINJA_C)
    capsule(px, w, h, hx - 1, hy + 1, hx + 4, hy - 1, 1.6, WOOD)
    disc(px, w, h, hx + 4, hy - 1, 2.6, GOLD)
    mx = (hx + tip[0]) * 0.5 + 3
    my = (hy + tip[1]) * 0.5 - 3
    capsule(px, w, h, hx + 5, hy - 2, mx, my, 2.0, STEEL)
    capsule(px, w, h, mx, my, tip[0], tip[1], 1.6, STEEL)
    pix(px, w, h, tip[0], tip[1], STEEL[1])
    pix(px, w, h, tip[0] - 1, tip[1], STEEL[0])


def star_icon(px, w, h, cx, cy):
    col = (232, 196, 72)
    for dx, dy in ((0, -3), (0, -2), (0, -1), (0, 0), (0, 1), (0, 2), (0, 3), (-3, 0), (-2, 0), (-1, 0), (1, 0), (2, 0), (3, 0), (-2, -2), (2, -2), (-2, 2), (2, 2)):
        pix(px, w, h, cx + dx, cy + dy, col)
    pix(px, w, h, cx, cy, (255, 244, 200))


def draw_actor(kind, pose):
    if kind == "yamo":
        cell, anchor, scale = 96, (48, 91), 1.05
        rad = dict(torso=16, thigh=8.2, shin=6.4, uarm=6.6, farm=5.4, head=8.2)
        cloth, pants, shoe_c = YAMO_C, YAMO_C, YAMO_C
    elif kind == "ninja":
        cell, anchor, scale = 64, (32, 61), 1.0
        rad = dict(torso=5.2, thigh=3.1, shin=2.3, uarm=2.3, farm=1.9, head=5.4)
        cloth, pants, shoe_c = NINJA_C, NINJA_C, NINJA_C
    else:
        cell, anchor, scale = 64, (32, 62), 1.0
        rad = dict(torso=5.6, thigh=3.4, shin=2.6, uarm=2.5, farm=2.0, head=5.6)
        cloth, pants, shoe_c = GI, PANTS, SHOE

    im = new(cell, cell)
    px = im.load()
    p = {k: rel(anchor, v, scale) for k, v in POSES[pose].items()}
    if kind == "yamo":
        # Wide sumo stance. Shared poses are built for the slim fighter.
        for key, shove in (("k1", 8), ("f1", 10), ("e1", 4), ("h1", 6), ("k2", -6), ("f2", -8)):
            p[key] = (p[key][0] + shove, p[key][1])
    hip, chest, head = p["hip"], p["chest"], p["head"]
    shoulder = ((chest[0] + hip[0]) / 2, (chest[1] + hip[1]) / 2 + 2)

    # Back arm and back leg.
    capsule(px, cell, cell, hip[0], hip[1], p["k2"][0], p["k2"][1], rad["thigh"], pants)
    capsule(px, cell, cell, p["k2"][0], p["k2"][1], p["f2"][0], p["f2"][1], rad["shin"], pants)
    if kind == "hero":
        shoe(px, cell, cell, p["f2"][0], p["f2"][1], shoe_c)
    else:
        disc(px, cell, cell, p["f2"][0], p["f2"][1] + 1, rad["shin"] + 0.8, shoe_c)
    sleeve = cloth
    capsule(px, cell, cell, shoulder[0], shoulder[1], p["e2"][0], p["e2"][1], rad["uarm"], sleeve)
    capsule(px, cell, cell, p["e2"][0], p["e2"][1], p["h2"][0], p["h2"][1], rad["farm"], sleeve if kind != "hero" else GI)
    if kind == "hero":
        fist(px, cell, cell, p["h2"][0], p["h2"][1], SKIN)
    elif kind == "yamo":
        disc(px, cell, cell, p["h2"][0], p["h2"][1], rad["farm"] + 1.4, YAMO_C)

    # Torso. Sumo gets a belly. The fighter gets a gi hem and a black belt.
    capsule(px, cell, cell, hip[0], hip[1] - 2, chest[0], chest[1], rad["torso"], cloth)
    if kind == "yamo":
        pass
    elif kind == "hero":
        capsule(px, cell, cell, hip[0] - 4, hip[1] - 1, hip[0] + 7, hip[1] + 6, 4.6, GI)
        capsule(px, cell, cell, hip[0] - 6, hip[1] + 1, hip[0] + 7, hip[1] + 2, 2.0, BELT)
        # Black stripe down the lead side of the jacket, the movie-suit cue.
        capsule(px, cell, cell, chest[0] + 3, chest[1] + 2, hip[0] + 5, hip[1] + 4, 1.15, BELT)
    else:
        capsule(px, cell, cell, hip[0] - 6, hip[1], hip[0] + 6, hip[1], 1.7, RED)

    # Front leg, then the lead arm so the punch and kick read in front.
    capsule(px, cell, cell, hip[0], hip[1], p["k1"][0], p["k1"][1], rad["thigh"], pants)
    capsule(px, cell, cell, p["k1"][0], p["k1"][1], p["f1"][0], p["f1"][1], rad["shin"], pants)
    if kind == "hero":
        shoe(px, cell, cell, p["f1"][0], p["f1"][1], shoe_c)
        if pose == "kick2":
            # The flying foot is a black shoe at the end of the extended leg.
            shoe(px, cell, cell, p["f1"][0], p["f1"][1] - 1, shoe_c)
    else:
        disc(px, cell, cell, p["f1"][0], p["f1"][1] + 1, rad["shin"] + 1.0, shoe_c)
    if kind == "yamo":
        disc(px, cell, cell, hip[0] + 1, hip[1] - 6, 18, YAMO_C)
        capsule(px, cell, cell, hip[0] - 16, hip[1] + 2, hip[0] + 16, hip[1] + 2, 5.0, RED)
        disc(px, cell, cell, hip[0] + 10, hip[1] + 1, 4.0, RED)
        pix(px, cell, cell, hip[0] + 10, hip[1], (255, 236, 210))
        pix(px, cell, cell, hip[0] + 11, hip[1] + 2, (255, 220, 180))
    capsule(px, cell, cell, shoulder[0], shoulder[1], p["e1"][0], p["e1"][1], rad["uarm"], sleeve)
    capsule(px, cell, cell, p["e1"][0], p["e1"][1], p["h1"][0], p["h1"][1], rad["farm"], sleeve if kind != "hero" else GI)
    if kind == "hero":
        big = pose in ("punch1", "punch2", "idle0", "idle1")
        fist(px, cell, cell, p["h1"][0], p["h1"][1], SKIN, big=big)
        if pose == "punch1":
            # Knuckle flash so the straight punch reads at a glance.
            capsule(px, cell, cell, p["h1"][0] - 2, p["h1"][1], p["h1"][0] + 3, p["h1"][1], 1.2, ((255, 244, 220), (255, 252, 240), (230, 200, 160)))
    elif kind == "yamo":
        disc(px, cell, cell, p["h1"][0], p["h1"][1], rad["farm"] + 1.8, YAMO_C)

    if kind == "hero":
        capsule(px, cell, cell, chest[0], chest[1] - 1, head[0], head[1] + 5, 2.0, SKIN)
        hero_head(px, cell, cell, head[0], head[1])
    elif kind == "ninja":
        ninja_head(px, cell, cell, head[0], head[1])
        saber(px, cell, cell, p["h1"][0], p["h1"][1], pose)
        if pose == "throw0":
            star_icon(px, cell, cell, p["h1"][0] + 1, p["h1"][1])
    else:
        yamo_head(px, cell, cell, head[0], head[1])
        capsule(px, cell, cell, chest[0], chest[1] - 2, head[0], head[1] + 8, 4.0, YAMO_C)

    outline(im)
    return im


def sheet(kind, order, cols):
    sample = draw_actor(kind, order[0])
    cw, ch = sample.size
    rows = (len(order) + cols - 1) // cols
    im = new(cols * cw, rows * ch)
    for i, name in enumerate(order):
        fr = draw_actor(kind, name)
        im.alpha_composite(fr, ((i % cols) * cw, (i // cols) * ch))
    return im


def draw_tiles():
    im = new(16 * 10, 16)
    px = im.load()

    def shade_rect(x0, y0, x1, y1, col, brick=False):
        mid, hi, sh = col
        for y in range(y0, y1):
            for x in range(x0, x1):
                n = ((x * 17 + y * 31) ^ (x * y)) & 7
                rgb = hi if y == y0 or (brick and y % 4 == 0 and n > 5) else sh if y == y1 - 1 else mid
                if n == 0:
                    rgb = tuple(max(0, c - 18) for c in rgb)
                elif n == 1:
                    rgb = tuple(min(255, c + 14) for c in rgb)
                px[x, y] = (*rgb, 255)

    # 0 floor top
    shade_rect(0, 4, 16, 16, STONE, True)
    for x in range(16):
        px[x, 3] = (*STONE[1], 255)
        px[x, 2] = (70, 120, 64, 255) if (x % 5) in (0, 1) else (*STONE[1], 255)
        if (x * 3) % 7 == 0:
            px[x, 1] = (86, 150, 72, 255)
    # 1 floor fill
    shade_rect(16, 0, 32, 16, STONE_DK, True)
    # 2 platform
    shade_rect(32, 2, 48, 12, STONE)
    for x in range(32, 48):
        px[x, 1] = (*STONE[1], 255)
        px[x, 2] = (78, 140, 78, 255) if x % 4 == 0 else (*STONE[1], 255)
        px[x, 12] = (*STONE[2], 255)
        px[x, 13] = (40, 36, 44, 255)
    # 3 ladder
    for y in range(16):
        px[16 * 3 + 2, y] = (*WOOD[0], 255)
        px[16 * 3 + 3, y] = (*WOOD[1], 255)
        px[16 * 3 + 12, y] = (*WOOD[0], 255)
        px[16 * 3 + 13, y] = (*WOOD[2], 255)
        if y % 4 == 1:
            for x in range(3, 13):
                px[16 * 3 + x, y] = (*WOOD[0], 255)
                if y + 1 < 16:
                    px[16 * 3 + x, y + 1] = (*WOOD[1], 255)
    # 4 vine
    for y in range(16):
        wob = 1 if (y // 3) % 2 == 0 else 0
        px[16 * 4 + 7 + wob, y] = (*VINE_C[2], 255)
        px[16 * 4 + 8 + wob, y] = (*VINE_C[0], 255)
        px[16 * 4 + 9 + wob, y] = (*VINE_C[1], 255)
        if y % 5 == 2:
            disc(px, 160, 16, 16 * 4 + 4 + wob, y, 2.2, LEAF)
            disc(px, 160, 16, 16 * 4 + 12 + wob, min(15, y + 1), 2.0, LEAF)
    # 5 electric off
    shade_rect(80, 6, 96, 16, ((70, 74, 92), (110, 116, 136), (40, 42, 56)))
    for x in range(80, 96, 3):
        px[x, 8] = (150, 156, 170, 255)
        px[x, 10] = (40, 44, 60, 255)
    # 6 electric warn
    shade_rect(96, 6, 112, 16, ((90, 70, 48), (160, 120, 60), (50, 36, 24)))
    for x in range(96, 112):
        if x % 2 == 0:
            px[x, 4] = (255, 210, 80, 255)
            px[x, 8] = (255, 160, 40, 255)
    # 7 electric on
    shade_rect(112, 6, 128, 16, ((40, 80, 150), (120, 190, 255), (20, 30, 80)))
    for y in range(2, 14):
        for x in range(112, 128):
            if (x + y * 3) % 4 == 0:
                px[x, y] = (230, 250, 255, 255)
            elif (x * 2 + y) % 5 == 0:
                px[x, y] = (80, 220, 255, 255)
    # 8 pedestal
    shade_rect(128, 0, 144, 16, STONE, True)
    for x in range(128, 144):
        px[x, 0] = (*GOLD[0], 255)
        px[x, 1] = (*STONE[1], 255)
    # 9 sealed crack
    shade_rect(144, 0, 160, 16, STONE_DK, True)
    for i in range(4, 12):
        px[144 + i, i] = (24, 18, 20, 255)
        px[145 + i, i] = (90, 70, 60, 255)
    return im


def draw_lanterns():
    im = new(32 * 4, 40)
    for f in range(4):
        ox = f * 32
        px = im.load()
        # String.
        for y in range(0, 8):
            px[ox + 16, y] = (40, 32, 28, 255)
        body_top = 8
        for y in range(0, 18):
            k = abs(y - 9) / 9
            half = int(8 - k * 3)
            yy = body_top + y
            for x in range(16 - half, 16 + half):
                rib = (x + f) % 4 == 0
                if y < 2 or y > 15:
                    col = (120, 48, 36, 255)
                elif rib:
                    col = (150, 64, 40, 255)
                else:
                    glow = 180 + ((f + y) % 3) * 20
                    col = (255, glow, 70, 255)
                px[ox + x, yy] = col
            # Highlight.
            hx = 16 - half + 2
            if 0 <= hx < 32 and 2 < y < 14:
                px[ox + hx, yy] = (255, 240, 180, 255)
        # Caps.
        for x in range(10, 22):
            px[ox + x, 7] = (70, 40, 28, 255)
            px[ox + x, 26] = (70, 40, 28, 255)
        # Tassel.
        px[ox + 16, 27] = (*GOLD[0], 255)
        px[ox + 16, 28] = (*RED[0], 255)
        px[ox + 15, 29] = (*RED[0], 255)
        px[ox + 16, 29] = (*RED[1], 255)
        px[ox + 17, 29] = (*RED[0], 255)
        px[ox + 16, 30 + (f % 2)] = (*RED[2], 255)
    return im


def draw_torch():
    im = new(16 * 4, 32)
    px = im.load()
    for f in range(4):
        ox = f * 16
        for y in range(14, 32):
            px[ox + 7, y] = (*WOOD[2], 255)
            px[ox + 8, y] = (*WOOD[0], 255)
        # Bowl.
        for x in range(5, 12):
            px[ox + x, 14] = (70, 60, 58, 255)
            px[ox + x, 15] = (100, 86, 70, 255)
        flames = [
            [(8, 12), (7, 10), (8, 9), (9, 10), (8, 7), (6, 11), (10, 11)],
            [(8, 11), (7, 9), (9, 8), (8, 6), (6, 12), (10, 10), (8, 13)],
            [(8, 12), (6, 9), (8, 7), (10, 9), (7, 6), (9, 11), (8, 4)],
            [(7, 11), (8, 8), (9, 10), (8, 5), (10, 12), (6, 10), (8, 13)],
        ]
        for x, y in flames[f]:
            px[ox + x, y] = (255, 220, 80, 255)
        for x, y in flames[f][:3]:
            px[ox + x, y + 1] = (240, 100, 36, 255)
        px[ox + 8, 8] = (255, 250, 210, 255)
    return im


def draw_statue():
    im = new(64, 72)
    px = im.load()
    # Plinth is separate. Sitting stone ox, side view.
    capsule(px, 64, 72, 16, 48, 46, 44, 12, STONE)  # body
    capsule(px, 64, 72, 40, 36, 56, 32, 8, STONE)  # head
    capsule(px, 64, 72, 50, 24, 60, 18, 2.2, STONE)  # horn
    capsule(px, 64, 72, 48, 22, 54, 14, 2.0, STONE)
    capsule(px, 64, 72, 18, 52, 16, 66, 4, STONE)
    capsule(px, 64, 72, 30, 54, 30, 67, 4.2, STONE)
    capsule(px, 64, 72, 42, 52, 44, 66, 3.8, STONE)
    disc(px, 64, 72, 54, 34, 2.0, ((20, 16, 18), (40, 30, 28), (10, 8, 10)))
    # Snout.
    capsule(px, 64, 72, 56, 36, 62, 40, 3.0, STONE)
    outline(im)
    return im


def draw_gate():
    """Two frames: closed, open. 48x88."""
    im = new(48 * 2, 88)

    def posts(px, ox):
        for y in range(16, 88):
            for x in range(4, 12):
                col = STONE[1] if x < 7 else STONE[0]
                px[ox + x, y] = (*col, 255)
            for x in range(36, 44):
                col = STONE[1] if x < 39 else STONE[0]
                px[ox + x, y] = (*col, 255)
        # Lintel and arch.
        for x in range(4, 44):
            px[ox + x, 16] = (*STONE[1], 255)
            px[ox + x, 17] = (*GOLD[0], 255)
            px[ox + x, 18] = (*STONE[0], 255)
        for y in range(8, 18):
            half = int((y - 8) * 1.6)
            for x in range(24 - half, 24 + half):
                if 4 <= x < 44 and px[ox + x, y][3] == 0:
                    px[ox + x, y] = (*STONE[2], 255)

    px = im.load()
    posts(px, 0)
    # Closed doors.
    for y in range(20, 86):
        for x in range(12, 36):
            plank = WOOD[0] if ((x // 6) % 2 == 0) else WOOD[2]
            if y % 18 in (0, 1):
                plank = (90, 70, 48)
            px[x, y] = (*plank, 255)
    for y in range(40, 48):
        px[30, y] = (*GOLD[1], 255)
        px[31, y] = (*GOLD[0], 255)
    disc(px, 96, 88, 28, 52, 3, GOLD)
    posts(px, 48)
    # Open: dark passage between posts.
    for y in range(20, 88):
        for x in range(12, 36):
            v = 18 + (y // 12)
            px[48 + x, y] = (v, v - 4, v + 6, 255)
    outline_keep = im.copy()
    # Don't outline the open darkness into a blob; outline only the closed frame edges manually is enough.
    return im


def draw_hatch():
    im = new(112 * 2, 36)
    px = im.load()
    for x in range(8, 104):
        for y in range(6, 32):
            px[x, y] = (*STONE_DK[0], 255)
            if y % 6 == 0:
                px[x, y] = (*STONE_DK[2], 255)
    # Circular seal.
    disc(px, 224, 36, 56, 18, 12, STONE)
    disc(px, 224, 36, 56, 18, 7, GOLD)
    disc(px, 224, 36, 56, 18, 3, ((40, 24, 20), (70, 40, 32), (20, 12, 10)))
    # Open hole.
    for x in range(120, 208):
        for y in range(8, 34):
            px[x, y] = (8, 6, 12, 255)
    disc(px, 224, 36, 164, 22, 14, GOLD)
    disc(px, 224, 36, 164, 22, 10, ((8, 6, 12), (20, 14, 18), (4, 2, 6)))
    for i in range(5):
        px[164, 16 + i] = (255, 210, 120, 255)
        px[160 + i, 24] = (255, 180, 70, 255)
    return im


def gradient(px, w, h, stops):
    """stops: list of (y, rgb)."""
    for y in range(h):
        for i in range(len(stops) - 1):
            y0, c0 = stops[i]
            y1, c1 = stops[i + 1]
            if y0 <= y <= y1:
                t = (y - y0) / max(1, (y1 - y0))
                # Bayer dither.
                bayer = ((0, 8, 2, 10), (12, 4, 14, 6), (3, 11, 1, 9), (15, 7, 13, 5))
                jit = (bayer[y & 3][0 & 3] - 7.5) / 16.0  # x applied inside
                for x in range(w):
                    j = (bayer[y & 3][x & 3] - 7.5) / 18.0
                    tt = min(1.0, max(0.0, t + j * 0.18))
                    rgb = tuple(int(c0[k] + (c1[k] - c0[k]) * tt) for k in range(3))
                    px[x, y] = (*rgb, 255)
                break


def fill_rect(px, x0, y0, x1, y1, rgb, w, h):
    for y in range(max(0, y0), min(h, y1)):
        for x in range(max(0, x0), min(w, x1)):
            px[x, y] = (*rgb, 255)


def noise_tint(px, x0, y0, x1, y1, amt, w, h):
    for y in range(max(0, y0), min(h, y1)):
        for x in range(max(0, x0), min(w, x1)):
            n = ((x * 374761393 + y * 668265263) ^ (x << 4)) & 255
            if n < 18:
                r, g, b, a = px[x, y]
                d = amt if n < 9 else -amt
                px[x, y] = (max(0, min(255, r + d)), max(0, min(255, g + d)), max(0, min(255, b + d)), a)


def roof(px, cx, base_y, half_w, rise, rgb, w, h):
    for x in range(-half_w, half_w):
        edge = 1 - abs(x) / half_w
        y_top = int(base_y - rise * edge)
        # Upturned eave.
        if abs(x) > half_w - 8:
            y_top -= int((abs(x) - (half_w - 8)) * 0.8)
        for y in range(y_top, base_y):
            if 0 <= cx + x < w and 0 <= y < h:
                shade = 18 if y == y_top else 0
                px[cx + x, y] = (min(255, rgb[0] + shade), min(255, rgb[1] + shade // 2), rgb[2], 255)


def draw_bg_court():
    w, h = 640, 336
    im = new(w, h)
    px = im.load()
    gradient(px, w, h, [
        (0, (16, 14, 42)),
        (120, (48, 28, 72)),
        (190, (150, 78, 72)),
        (230, (210, 140, 90)),
        (336, (54, 42, 52)),
    ])
    # Moon.
    disc(px, w, h, 86, 48, 16, ((232, 220, 196), (255, 248, 230), (190, 176, 150)))
    disc(px, w, h, 80, 44, 3, ((200, 188, 160), (220, 210, 184), (170, 160, 140)))
    # Mountains.
    for i, peak in enumerate((40, 90, 150, 40, 110, 70, 130, 60)):
        cx = 20 + i * 84
        col = (42, 32, 64) if i % 2 == 0 else (58, 36, 62)
        roof(px, cx, 214, 70, peak // 3, col, w, h)
    # Far pagodas.
    for cx, sc in ((150, 1.0), (300, 0.7), (470, 1.15)):
        roof(px, cx, 168, int(54 * sc), int(22 * sc), (72, 28, 36), w, h)
        roof(px, cx, 188, int(40 * sc), int(16 * sc), (90, 32, 40), w, h)
        fill_rect(px, cx - int(10 * sc), 188, cx + int(10 * sc), 230, (48, 28, 36), w, h)
    # Fortress wall sits low so the roofs stay visible.
    fill_rect(px, 0, 208, 640, 300, (78, 70, 78), w, h)
    noise_tint(px, 0, 208, 640, 300, 16, w, h)
    for y in range(216, 292, 8):
        for x in range(0, 640):
            if ((x // 16) + (y // 8)) % 2 == 0 and x % 16 == 0:
                if px[x, y][3]:
                    r, g, b, a = px[x, y]
                    px[x, y] = (max(0, r - 20), max(0, g - 20), max(0, b - 16), a)
    for x in range(6, 640, 28):
        fill_rect(px, x, 192, x + 16, 212, (96, 88, 96), w, h)
        fill_rect(px, x, 192, x + 16, 194, (140, 128, 120), w, h)
        if x % 56 == 6:
            px[x + 8, 202] = (86, 140, 78, 255)
    # Narrow windows.
    for wx in (80, 180, 340, 460):
        fill_rect(px, wx, 230, wx + 10, 252, (28, 22, 32), w, h)
        fill_rect(px, wx + 3, 234, wx + 7, 244, (232, 170, 80), w, h)
    disc(px, w, h, 250, 248, 20, STONE)
    disc(px, w, h, 250, 248, 12, ((48, 36, 40), (80, 60, 56), (24, 18, 22)))
    disc(px, w, h, 250, 248, 4, GOLD)
    fill_rect(px, 568, 214, 636, 304, (28, 22, 32), w, h)
    for x in range(560, 640):
        px[x, 210] = (*STONE[1], 255)
        px[x, 211] = (*GOLD[0], 255)
    for bx in (130, 420):
        fill_rect(px, bx, 212, bx + 12, 268, (150, 32, 40), w, h)
        fill_rect(px, bx, 212, bx + 12, 216, (220, 180, 70), w, h)
        fill_rect(px, bx + 4, 258, bx + 8, 272, (220, 180, 70), w, h)
    # Courtyard darkening toward the floor.
    for y in range(250, 336):
        t = (y - 250) / 86
        for x in range(w):
            r, g, b, a = px[x, y]
            k = 1 - 0.45 * t
            px[x, y] = (int(r * k), int(g * k), int(b * k), a)
    return im


def draw_bg_hall():
    w, h = 640, 336
    im = new(w, h)
    px = im.load()
    gradient(px, w, h, [
        (0, (22, 16, 32)),
        (160, (36, 24, 40)),
        (336, (18, 16, 24)),
    ])
    # Ceiling beams.
    for y in (16, 34):
        fill_rect(px, 0, y, 640, y + 8, (48, 32, 36), w, h)
    # Frieze.
    fill_rect(px, 0, 52, 640, 70, (72, 40, 44), w, h)
    for x in range(8, 640, 18):
        disc(px, w, h, x, 60, 4, GOLD)
    # Pillars.
    for cx in (70, 210, 430, 570):
        fill_rect(px, cx, 70, cx + 22, 300, (128, 36, 40), w, h)
        fill_rect(px, cx, 70, cx + 6, 300, (168, 64, 60), w, h)
        fill_rect(px, cx - 4, 66, cx + 26, 78, (180, 140, 60), w, h)
        fill_rect(px, cx - 4, 286, cx + 26, 300, (90, 28, 32), w, h)
    # Banners between pillars.
    for bx in (120, 300, 500):
        fill_rect(px, bx, 78, bx + 16, 150, (40, 48, 92), w, h)
        fill_rect(px, bx, 78, bx + 16, 84, (220, 180, 80), w, h)
    noise_tint(px, 0, 70, 640, 310, 12, w, h)
    for y in range(280, 336):
        t = (y - 280) / 56
        for x in range(w):
            r, g, b, a = px[x, y]
            k = 1 - 0.5 * t
            px[x, y] = (int(r * k), int(g * k), int(b * k), a)
    return im


def draw_bg_gallery():
    w, h = 640, 336
    im = new(w, h)
    px = im.load()
    gradient(px, w, h, [
        (0, (8, 12, 36)),
        (140, (16, 28, 64)),
        (210, (28, 40, 78)),
        (336, (20, 24, 36)),
    ])
    # Stars.
    rng = 12345
    for i in range(48):
        rng = (rng * 1103515245 + 12345) & 0x7FFFFFFF
        x = rng % 640
        rng = (rng * 1103515245 + 12345) & 0x7FFFFFFF
        y = rng % 150
        px[x, y] = (230, 230, 220, 255) if i % 4 else (180, 190, 210, 255)
    disc(px, w, h, 540, 46, 11, ((210, 216, 230), (245, 248, 255), (160, 170, 190)))
    # Ridges.
    for i, peak in enumerate((30, 50, 36, 64, 40, 55, 28)):
        roof(px, 30 + i * 100, 200, 80, peak, (16, 22, 48) if i % 2 == 0 else (22, 30, 58), w, h)
    # Towers.
    for tx in (0, 600):
        fill_rect(px, tx, 120, tx + 40, 300, (50, 52, 68), w, h)
        roof(px, tx + 20, 120, 28, 24, (70, 32, 40), w, h)
        fill_rect(px, tx + 14, 150, tx + 26, 170, (255, 180, 80), w, h)
    # Rail silhouettes.
    fill_rect(px, 40, 210, 600, 218, (42, 36, 40), w, h)
    for x in range(48, 600, 22):
        fill_rect(px, x, 200, x + 4, 230, (42, 36, 40), w, h)
    # Trench under the electric run.
    fill_rect(px, 200, 286, 440, 336, (10, 16, 32), w, h)
    for x in range(200, 440, 10):
        px[x, 286] = (80, 140, 180, 255)
    noise_tint(px, 0, 220, 640, 336, 10, w, h)
    return im


def main():
    sheet("hero", HERO_ORDER, 8).save(os.path.join(OUT, "hero.png"))
    sheet("ninja", NINJA_ORDER, 6).save(os.path.join(OUT, "ninja.png"))
    sheet("yamo", YAMO_ORDER, 5).save(os.path.join(OUT, "yamo.png"))
    draw_tiles().save(os.path.join(OUT, "tiles.png"))
    draw_lanterns().save(os.path.join(OUT, "lantern.png"))
    draw_torch().save(os.path.join(OUT, "torch.png"))
    draw_statue().save(os.path.join(OUT, "statue.png"))
    draw_gate().save(os.path.join(OUT, "gate.png"))
    draw_hatch().save(os.path.join(OUT, "hatch.png"))
    draw_bg_court().save(os.path.join(OUT, "bg0.png"))
    draw_bg_hall().save(os.path.join(OUT, "bg1.png"))
    draw_bg_gallery().save(os.path.join(OUT, "bg2.png"))

    # Contact sheet: guard, straight punch, side kick, flying kick, ninja slash, sumo.
    def blow(path, col, row, cell, scale=3):
        im = Image.open(path)
        fr = im.crop((col * cell, row * cell, (col + 1) * cell, (row + 1) * cell))
        return fr.resize((cell * scale, cell * scale), Image.Resampling.NEAREST)

    preview = Image.new("RGBA", (900, 420), (28, 22, 40, 255))
    # hero frames: 0 idle, 14 punch, 17 side kick, 18 flying kick. 8 columns.
    for i, idx in enumerate((0, 14, 17, 18)):
        preview.alpha_composite(blow(os.path.join(OUT, "hero.png"), idx % 8, idx // 8, 64), (16 + i * 200, 16))
    preview.alpha_composite(blow(os.path.join(OUT, "ninja.png"), 0, 0, 64), (16, 220))
    preview.alpha_composite(blow(os.path.join(OUT, "ninja.png"), 0, 1, 64), (220, 220))  # punch1 slash, frame 6
    preview.alpha_composite(blow(os.path.join(OUT, "yamo.png"), 0, 0, 96, 2), (440, 200))
    preview.alpha_composite(blow(os.path.join(OUT, "yamo.png"), 1, 1, 96, 2), (660, 200))  # slap
    preview.save(os.path.join(OUT, "preview.png"))
    print("assets written to", OUT)


if __name__ == "__main__":
    main()
