"""Playwright smoke test: plays Neon Knights from the title screen to the
ending through every scene, dies twice along the way, checks the PC-speaker
audio (created only after a user gesture; mute works and is remembered),
and screenshots each step. Run against a server hosting dist/, at the site
root or under a subpath such as GitHub Pages' /neon-knights/:

    python tools/smoke_test.py http://localhost:8765/
    SHOT_PREFIX=m3 python tools/smoke_test.py http://localhost:8766/neon-knights/
"""
import asyncio, os, sys
from playwright.async_api import async_playwright

URL = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8765/"
OUT = os.environ.get("SHOTS_DIR", "/workspace/neon-nights-shots")
PREFIX = os.environ.get("SHOT_PREFIX", "m3")

# Instrument Web Audio before the game loads: count user gestures, record
# how many gestures had happened when each AudioContext was created, and
# count oscillators (one per beep).
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
VERBS = ["Walk", "Look", "Take", "Talk", "Use"]

async def main():
    logs, errors = [], []
    os.makedirs(OUT, exist_ok=True)
    async with async_playwright() as p:
        b = await p.chromium.launch()
        page = await b.new_page(viewport={"width": 1280, "height": 800})
        page.on("console", lambda m: logs.append(f"{m.type}: {m.text}"))
        page.on("pageerror", lambda e: errors.append(str(e)))
        await page.add_init_script(AUDIO_PROBE)
        await page.goto(URL)
        await page.wait_for_timeout(1500)
        box = await page.locator("#game").bounding_box()
        checks = []
        def check(name, ok, detail=""):
            checks.append((name, bool(ok)))
            print(("PASS " if ok else "FAIL ") + name, detail)
        async def probe():
            return await page.evaluate("window.__probe")
        verb = [1]

        async def click_fb(x, y, button="left"):
            await page.mouse.click(box["x"] + (x + 0.5) * box["width"] / 320,
                                   box["y"] + (y + 0.5) * box["height"] / 200, button=button)
        async def set_verb(name):
            want = VERBS.index(name)
            while verb[0] != want:
                await page.keyboard.press("Tab")
                verb[0] = (verb[0] + 1) % len(VERBS)
        async def click(name, x, y, wait=6000):
            await set_verb(name)
            await click_fb(x, y)
            await page.wait_for_timeout(wait)
        async def drain(n=6):
            # Escape dismisses a message box; with none open it just clears input.
            for _ in range(n):
                await page.keyboard.press("Escape")
                await page.wait_for_timeout(120)
        async def cmd(text, settle=True):
            await page.keyboard.type(text, delay=10)
            await page.keyboard.press("Enter")
            await page.wait_for_timeout(250)
        async def shot(name):
            await page.locator("#screen").screenshot(path=f"{OUT}/{PREFIX}-{name}.png")
            print("shot", name)

        p0 = await probe()
        check("no AudioContext before a user gesture", not p0["created"], p0)
        await shot("01-title")
        await page.keyboard.press("Enter"); await page.wait_for_timeout(600)
        p1 = await probe()
        check("AudioContext created inside the first gesture", p1["created"] and p1["created"][0] >= 1, p1)
        check("title jingle beeps", p1["oscs"] >= 5, p1)
        await shot("02-message-box")
        await drain()
        await cmd("xyzzy")
        p2 = await probe()
        check("parser error buzzes", p2["oscs"] > p1["oscs"], p2)
        await shot("03-parser-error")
        await drain()
        # --- The Rusty Anchor
        await click("Take", 120, 79, 5000)
        p3 = await probe()
        check("score blip", p3["oscs"] > p2["oscs"], p3)
        await drain()
        await cmd("read matchbook"); await drain()
        await click("Use", 236, 85, 5000); await drain()
        await click("Talk", 151, 70, 5000); await drain()
        await cmd("drink the special"); await drain()
        await click("Walk", 278, 103, 6000)        # out the front door
        await shot("04-boardwalk-intro")
        await drain()
        # --- Boardwalk
        await click("Talk", 168, 108, 6000); await drain()
        await cmd("compliment dot"); await drain()
        await click("Use", 296, 130, 4000); await drain()
        await shot("05-boardwalk")
        await click("Walk", 2, 140, 9000)          # west edge -> alley
        await drain()
        # --- Alley
        await click("Walk", 226, 140, 4000)
        await cmd("give hot dog to mime"); await drain()
        await click("Walk", 170, 158, 4000)
        await cmd("use comb on grate"); await drain()
        await cmd("save"); await drain()
        # Death #1: the dumpster.
        await click("Walk", 118, 124, 4000)
        pd = await probe()
        await cmd("climb into dumpster")
        await drain(); await page.wait_for_timeout(400)
        p4 = await probe()
        check("death dirge", p4["oscs"] - pd["oscs"] >= 8, (pd, p4))
        await shot("06-death-dumpster")
        await page.keyboard.press("r"); await page.wait_for_timeout(500)
        await drain()
        await shot("07-restored-in-alley")
        await click("Walk", 318, 140, 5000)        # east edge -> boardwalk
        await drain()
        await click("Walk", 152, 103, 6000)        # into the Lucky Gull
        await drain()
        # --- Store
        await click("Talk", 60, 70, 7000); await drain()
        await cmd("buy carnation"); await drain()
        await cmd("buy mints"); await drain()
        await cmd("wear carnation"); await drain()
        await cmd("save"); await drain()
        # Death #2: the slushie, with sound muted from the tombstone (M key).
        await click("Use", 282, 70, 8000)
        await drain(); await page.wait_for_timeout(400)
        await shot("08-death-slushie")
        await page.keyboard.press("m"); await page.wait_for_timeout(200)
        muted = await page.evaluate("localStorage.getItem('neon-knights.sound')")
        check("M mutes and is remembered", muted == "off", muted)
        await page.keyboard.press("r"); await page.wait_for_timeout(500)
        await drain()
        pm = await probe()
        await cmd("frobnicate"); await drain()
        pm2 = await probe()
        check("muted: no beeps", pm2["oscs"] == pm["oscs"], (pm, pm2))
        await cmd("sound on")
        await shot("09-sound-on")
        await drain()
        unmuted = await page.evaluate("localStorage.getItem('neon-knights.sound')")
        check("'sound on' unmutes and is remembered", unmuted == "on", unmuted)
        await click("Walk", 160, 173, 6000)        # out to the boardwalk
        await drain()
        await click("Walk", 261, 103, 6000)        # into the casino
        await drain()
        # --- Casino
        await click("Walk", 238, 103, 6000)        # Tiny blocks the elevator
        await drain()
        await click("Talk", 238, 95, 6000); await drain()
        await click("Walk", 238, 103, 5000)        # ride up
        await drain()
        # --- Rooftop
        await cmd("eat mints"); await drain()
        pe = await probe()
        await click("Talk", 106, 110, 5000)
        await drain(8); await page.wait_for_timeout(600)
        pe2 = await probe()
        check("ending fanfare", pe2["oscs"] >= pe["oscs"] + 8, pe2)
        await shot("10-ending")
        # The ending offers Restore / Restart / Quit: click Restore.
        await click_fb(93, 187); await page.wait_for_timeout(500)
        await shot("11-ending-restore")
        await drain()
        save = await page.evaluate("localStorage.getItem('neon-knights.save.v1')")
        print("save:", save)
        # Legacy-key migration: an M1 save under the old key must still load.
        await page.evaluate("""() => {
            localStorage.removeItem('neon-knights.save.v1');
            localStorage.setItem('neon-nights.save.v1', 'NN1|13|7|cmd|236|112|9|0|0');
        }""")
        await page.reload(); await page.wait_for_timeout(1500)
        await page.keyboard.press("Enter"); await page.wait_for_timeout(300)
        await drain()
        await cmd("restore")
        await shot("12-legacy-save-restored")
        keys = await page.evaluate("[localStorage.getItem('neon-knights.save.v1'), localStorage.getItem('neon-nights.save.v1')]")
        print("after migration [new, old]:", keys)
        print("title:", await page.title())
        await b.close()
    print("console:", *logs, sep="\n  ")
    print("pageerrors:", errors)
    bad = [l for l in logs if l.startswith(("error", "warning"))] + errors
    failed = [n for n, ok in checks if not ok]
    print("checks failed:", failed)
    sys.exit(1 if bad or failed else 0)

asyncio.run(main())
