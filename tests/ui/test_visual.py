"""Visual regression: compare the game's own 320x200 framebuffer (palette
indices read through the test hook, expanded to RGB) with committed
baselines. No page screenshots are involved, so fonts, GPU and OS
rendering can't affect the result.

Update baselines on purpose with:  UPDATE_SNAPSHOTS=1 python -m pytest tests/ui -k visual
On failure, <name>-actual.png and <name>-diff.png are written to
tests/visual/output/.
"""
from __future__ import annotations

import os
import pathlib

import pytest
from PIL import Image, ImageChops, ImageDraw

from nk import SAVES, Game, Save

HERE = pathlib.Path(__file__).resolve().parent
BASELINE = HERE.parent / "visual" / "baseline"
OUTPUT = HERE.parent / "visual" / "output"
UPDATE = os.environ.get("UPDATE_SNAPSHOTS") == "1"
# Tolerances. The framebuffer is exact, so by default a single wrong pixel
# fails; COLOR_TOLERANCE only forgives tiny per-channel differences (there
# are none between palette colors, which differ by >= 0x55).
MAX_DIFF_PIXELS = int(os.environ.get("VISUAL_MAX_DIFF_PIXELS", "0"))
COLOR_TOLERANCE = int(os.environ.get("VISUAL_COLOR_TOLERANCE", "8"))


def diff_mask(a: Image.Image, b: Image.Image) -> Image.Image:
    d = ImageChops.difference(a, b).convert("RGB")
    r, g, bl = d.split()
    m = ImageChops.lighter(ImageChops.lighter(r, g), bl)
    return m.point(lambda v: 255 if v > COLOR_TOLERANCE else 0)


def check_frame(name: str, img: Image.Image) -> None:
    BASELINE.mkdir(parents=True, exist_ok=True)
    base_path = BASELINE / f"{name}.png"
    if UPDATE:
        img.save(base_path, optimize=True)
        return
    OUTPUT.mkdir(parents=True, exist_ok=True)
    if not base_path.exists():
        img.save(OUTPUT / f"{name}-actual.png")
        pytest.fail(f"no baseline for {name}; create it with UPDATE_SNAPSHOTS=1")
    base = Image.open(base_path).convert("RGB")
    assert base.size == img.size, f"{name}: size {img.size} != baseline {base.size}"
    mask = diff_mask(base, img)
    bad = mask.histogram()[255]
    if bad > MAX_DIFF_PIXELS:
        img.save(OUTPUT / f"{name}-actual.png")
        # baseline | actual | diff (dimmed baseline, changed pixels in red), 3x.
        dim = Image.blend(base, Image.new("RGB", base.size, (0, 0, 0)), 0.7)
        red = Image.new("RGB", base.size, (255, 0, 0))
        diff = Image.composite(red, dim, mask)
        # Ring the changed area so even a single pixel is easy to spot.
        x0, y0, x1, y1 = mask.getbbox()
        ImageDraw.Draw(diff).rectangle((x0 - 4, y0 - 4, x1 + 3, y1 + 3), outline=(255, 255, 0))
        sheet = Image.new("RGB", (base.width * 3 + 8, base.height), (64, 64, 64))
        for i, im in enumerate([base, img, diff]):
            sheet.paste(im, (i * (base.width + 4), 0))
        sheet = sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST)
        sheet.save(OUTPUT / f"{name}-diff.png")
        pytest.fail(
            f"{name}: {bad} pixel(s) differ from baseline (allowed {MAX_DIFF_PIXELS}), "
            f"bbox {mask.getbbox()}; see {OUTPUT / (name + '-diff.png')}"
        )


def test_visual_title(game: Game):
    check_frame("title", game.frame())


def test_visual_message_box(game: Game):
    game.key("Enter")
    assert game.state()["message"].startswith("Ocean Bluffs")
    check_frame("message-box", game.frame())


def test_visual_bar(game: Game):
    game.start()
    game.step(3)
    check_frame("scene-bar", game.frame())


def test_visual_parser_error(game: Game):
    game.start()
    game.type("xyzzy")
    check_frame("parser-error", game.frame())


@pytest.mark.parametrize("scene", ["boardwalk", "alley", "store", "casino", "rooftop"])
def test_visual_scene(game: Game, scene: str):
    st = game.restore(SAVES[scene])
    assert st["scene"].lower() == scene
    game.step(3)
    check_frame(f"scene-{scene}", game.frame())


def test_visual_death(game: Game):
    game.restore(SAVES["store"])
    game.walk(282, 113)
    game.type("drink slushie")
    game.messages()
    assert game.state()["mode"] == "Dead"
    check_frame("death-slushie", game.frame())


def test_visual_ending(game: Game):
    game.restore(Save("Rooftop", 50, SAVES["rooftop"].flags + ["AteMints"], "cmf", 0, (124, 134)))
    game.type("sing to vonda")
    game.messages()
    assert game.state()["mode"] == "Ending"
    check_frame("ending", game.frame())
