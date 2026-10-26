"""Helpers for driving Neon Knights in a browser through its test hooks.

The page is opened with `?test`, which freezes the game clock: time only
advances when a test calls `step()` / `until_idle()`, so every frame is
reproducible. Input goes through real Playwright keyboard and mouse events;
state and pixels are read back through the wasm-exported `test_*` hooks
(see src/web.rs).
"""
from __future__ import annotations

import json
from dataclasses import dataclass, field

from PIL import Image

W, H = 320, 200
VERBS = ["Walk", "Look", "Take", "Talk", "Use"]

# Mirrors of the save format in src/game/state.rs (NN3 records; NN2 and NN1
# are still readable, see Save.encode_nn2).
SCENES = ["Bar", "Boardwalk", "Alley", "Store", "Casino", "Rooftop", "Pier", "Laundromat", "Motel"]
HINT_SLOTS = 24
FLAG_BITS = {
    name: i
    for i, name in enumerate(
        "TookMatchbook ReadMatchbook PlayedJukebox GotDrink DrankDrink EggWarned "
        "SatOnStool SeenIntro VisitedBoardwalk VisitedAlley VisitedStore VisitedCasino "
        "VisitedRooftop GotHotDog UsedTelescope MimeFed GotFiver BoughtCarnation "
        "BoughtMints WearingCarnation TinyApproved AteMints Won "
        "Sang GotTicket DeeBriefed GotRod GotShrimp CaughtFlounder GotKey GotGown "
        "VisitedPier VisitedLaundromat VisitedMotel DryerOpen FerrisAway".split()
    )
}

# Audio instrumentation, installed before the page loads: counts user
# gestures, records how many had happened when each AudioContext was made,
# and counts oscillators (one per beep).
AUDIO_PROBE = """
(() => {
  const probe = { gestures: 0, created: [], oscs: 0 };
  window.__probe = probe;
  for (const ev of ['keydown', 'mousedown'])
    window.addEventListener(ev, () => probe.gestures++, true);
  const Orig = window.AudioContext;
  window.AudioContext = class extends Orig {
    constructor(...a) { super(...a); probe.created.push(probe.gestures); }
    createOscillator() { probe.oscs++; return super.createOscillator(); }
  };
})();
"""


@dataclass
class Save:
    """Build a save record to jump straight to a point in the game."""

    scene: str
    score: int
    flags: list[str] = field(default_factory=list)
    inv: str = "qcm"
    cash: int = 0
    hero: tuple[int, int] = (40, 150)

    def _bits(self) -> int:
        bits = 0
        for f in self.flags:
            bits |= 1 << FLAG_BITS[f]
        return bits

    def encode(self) -> str:
        """The current NN3 format: nine talk counters and 24 hint digits."""
        x, y = self.hero
        talks = ",".join(["0"] * 9)
        return (
            f"NN3|{SCENES.index(self.scene)}|{self.score}|{self._bits()}|{self.inv}|{self.cash}"
            f"|{x}|{y}|1|{talks}|0|{'0' * HINT_SLOTS}"
        )

    def encode_nn2(self) -> str:
        """The Milestone 3 format (six talk counters, no hints), for load tests."""
        x, y = self.hero
        return (
            f"NN2|{SCENES.index(self.scene)}|{self.score}|{self._bits()}|{self.inv}|{self.cash}"
            f"|{x}|{y}|1|0,0,0,0,0,0|0"
        )


BAR_DONE = ["TookMatchbook", "ReadMatchbook", "PlayedJukebox", "GotDrink", "DrankDrink", "SeenIntro", "VisitedBoardwalk"]
SAVES = {
    "boardwalk": Save("Boardwalk", 20, BAR_DONE, "cm", 0, (51, 110)),
    "alley": Save("Alley", 25, BAR_DONE + ["GotHotDog", "UsedTelescope", "VisitedAlley"], "cmh", 0, (300, 140)),
    "store": Save(
        "Store", 35,
        BAR_DONE + ["GotHotDog", "UsedTelescope", "VisitedAlley", "MimeFed", "GotFiver", "VisitedStore"],
        "cm", 5, (160, 160),
    ),
    "casino": Save(
        "Casino", 41,
        BAR_DONE + ["GotHotDog", "UsedTelescope", "VisitedAlley", "MimeFed", "GotFiver", "VisitedStore",
                    "BoughtCarnation", "BoughtMints", "VisitedCasino"],
        "cmfn", 0, (160, 160),
    ),
    "rooftop": Save(
        "Rooftop", 48,
        BAR_DONE + ["GotHotDog", "UsedTelescope", "VisitedAlley", "MimeFed", "GotFiver", "VisitedStore",
                    "BoughtCarnation", "BoughtMints", "VisitedCasino", "WearingCarnation", "TinyApproved",
                    "VisitedRooftop"],
        "cmfn", 0, (154, 112),
    ),
}


def pelican_x(ticks: int) -> int:
    """Percival's x on the pier (mirrors world::pier::pelican_pos)."""
    span, pace, x0 = 110, 6, 122
    t = (ticks // pace) % (2 * span)
    return x0 + (t if t < span else 2 * span - t)


_SANG = SAVES["rooftop"].flags + ["AteMints", "Sang", "GotTicket"]
_CHAIN = ["DeeBriefed", "GotRod", "GotShrimp", "FerrisAway", "VisitedPier", "VisitedLaundromat", "VisitedMotel"]
SAVES.update(
    {
        # Just sang for the ticket (62/80); the pier chain is all ahead.
        "sang": Save("Rooftop", 62, _SANG, "cmft", 0, (124, 134)),
        "laundromat": Save("Laundromat", 62, _SANG + ["VisitedLaundromat"], "cmft", 0, (300, 140)),
        "pier": Save("Pier", 66, _SANG + ["DeeBriefed", "GotRod", "VisitedLaundromat", "VisitedPier"], "cmfr", 0, (16, 128)),
        "motel": Save("Motel", 66, _SANG + ["DeeBriefed", "GotRod", "VisitedLaundromat", "VisitedPier", "VisitedMotel"], "cmfr", 0, (16, 128)),
        # Key in hand, dryer still locked (75/80).
        "dryer": Save("Laundromat", 75, _SANG + _CHAIN + ["CaughtFlounder", "GotKey"], "cmfrk", 0, (300, 140)),
        # Gown in hand, one step from the ending (78/80).
        "gown": Save("Rooftop", 78, _SANG + _CHAIN + ["CaughtFlounder", "GotKey", "DryerOpen", "GotGown"], "cmfrkg", 0, (124, 134)),
    }
)


class Game:
    def __init__(self, page, url: str):
        self.page = page
        self.url = url
        self.box = None

    # ------------------------------------------------------------ setup
    def open(self, probe: bool = False) -> "Game":
        if probe:
            self.page.add_init_script(AUDIO_PROBE)
        sep = "&" if "?" in self.url else "?"
        self.page.goto(self.url + sep + "test")
        self.page.wait_for_function("typeof window.wasmBindings?.test_state === 'function'")
        self.box = self.page.locator("#game").bounding_box()
        return self

    def reload(self) -> "Game":
        self.page.reload()
        self.page.wait_for_function("typeof window.wasmBindings?.test_state === 'function'")
        self.box = self.page.locator("#game").bounding_box()
        return self

    # ------------------------------------------------------------ state
    def state(self) -> dict:
        return json.loads(self.page.evaluate("wasmBindings.test_state()"))

    def probe(self) -> dict:
        return self.page.evaluate("window.__probe")

    def storage(self, key: str):
        return self.page.evaluate("k => localStorage.getItem(k)", key)

    def step(self, n: int) -> None:
        self.page.evaluate("n => wasmBindings.test_step(n)", n)

    def until_idle(self, max_ticks: int = 4000) -> int:
        n = self.page.evaluate("n => wasmBindings.test_until_idle(n)", max_ticks)
        assert n < max_ticks, "hero never arrived"
        return n

    # ------------------------------------------------------------ input
    def key(self, k: str) -> None:
        self.page.keyboard.press(k)

    def type(self, text: str) -> dict:
        """Type a command at the prompt and press Enter; returns the new state."""
        assert self.state()["message"] is None, "a message box is in the way"
        self.page.keyboard.type(text)
        self.page.keyboard.press("Enter")
        return self.state()

    def messages(self) -> list[str]:
        """Read and dismiss every queued message box, in order."""
        out = []
        for _ in range(40):
            m = self.state()["message"]
            if m is None:
                return out
            out.append(m)
            self.key("Escape")
        raise AssertionError("message boxes never ran out")

    def click_fb(self, x: int, y: int, button: str = "left") -> None:
        b = self.box
        self.page.mouse.click(b["x"] + (x + 0.5) * b["width"] / W, b["y"] + (y + 0.5) * b["height"] / H, button=button)

    def set_verb(self, name: str) -> None:
        for _ in range(len(VERBS)):
            if self.state()["verb"] == name:
                return
            self.key("Tab")
        raise AssertionError(f"verb {name} not found")

    def act(self, verb: str, x: int, y: int) -> dict:
        """Click (x, y) with a verb; walk there (and do it) deterministically."""
        self.set_verb(verb)
        self.click_fb(x, y)
        self.until_idle()
        return self.state()

    def walk(self, x: int, y: int) -> dict:
        return self.act("Walk", x, y)

    def travel(self, *places: str) -> list[str]:
        """Type "go to <place>" for each place in turn, walking to the exit
        each time; returns the arrival messages (dismissed)."""
        out = []
        for p in places:
            self.type(f"go to {p}")
            self.until_idle()
            out += self.messages()
        return out

    # ------------------------------------------------------------ flows
    def start(self) -> list[str]:
        """Title -> playing; returns the intro messages (dismissed)."""
        assert self.state()["mode"] == "Title"
        self.key("Enter")
        return self.messages()

    def restore(self, save: Save | str) -> dict:
        data = save.encode() if isinstance(save, Save) else save
        self.page.evaluate("d => localStorage.setItem('neon-knights.save.v1', d)", data)
        st = self.state()
        if st["mode"] == "Title":
            self.start()
        self.type("restore")
        msgs = self.messages()
        assert msgs and msgs[0].startswith("Game restored"), msgs
        return self.state()

    # ------------------------------------------------------------ pixels
    def frame(self) -> Image.Image:
        """The composed 320x200 framebuffer, palette-expanded to RGB."""
        idx, pal = self.page.evaluate(
            "() => [Array.from(wasmBindings.test_framebuffer()), Array.from(wasmBindings.test_palette())]"
        )
        assert len(idx) == W * H
        img = Image.new("P", (W, H))
        img.putpalette(pal + [0] * (768 - len(pal)))
        img.putdata(idx)
        return img.convert("RGB")
