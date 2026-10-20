"""Playwright smoke test: plays Neon Knights from the title screen to the
ending through every scene, dies once along the way, and screenshots each
scene. Run against a server hosting dist/:

    python tools/smoke_test.py http://localhost:8765/
"""
import asyncio, os, sys
from playwright.async_api import async_playwright

URL = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8765/"
OUT = os.environ.get("SHOTS_DIR", "/workspace/neon-nights-shots")
VERBS = ["Walk", "Look", "Take", "Talk", "Use"]

async def main():
    logs, errors = [], []
    os.makedirs(OUT, exist_ok=True)
    async with async_playwright() as p:
        b = await p.chromium.launch()
        page = await b.new_page(viewport={"width": 1280, "height": 800})
        page.on("console", lambda m: logs.append(f"{m.type}: {m.text}"))
        page.on("pageerror", lambda e: errors.append(str(e)))
        await page.goto(URL)
        await page.wait_for_timeout(1500)
        box = await page.locator("#hi").bounding_box()
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
            await page.locator("#screen").screenshot(path=f"{OUT}/{name}.png")
            print("shot", name)

        await shot("m2-01-title")
        await page.keyboard.press("Enter"); await page.wait_for_timeout(400)
        await drain()
        # --- The Rusty Anchor
        await click("Take", 120, 79, 5000); await drain()
        await cmd("read matchbook"); await drain()
        await click("Use", 236, 85, 5000); await drain()
        await click("Talk", 151, 70, 5000); await drain()
        await cmd("drink the special"); await drain()
        await click("Walk", 278, 103, 6000)        # out the front door
        await shot("m2-02-boardwalk-intro")
        await drain()
        # --- Boardwalk
        await click("Talk", 168, 108, 6000); await drain()
        await cmd("compliment dot"); await drain()
        await click("Use", 296, 130, 4000); await drain()
        await shot("m2-03-boardwalk")
        await click("Walk", 2, 140, 9000)          # west edge -> alley
        await drain()
        # --- Alley
        await shot("m2-04-alley")
        await click("Walk", 226, 140, 4000)
        await cmd("give hot dog to mime"); await drain()
        await click("Walk", 170, 158, 4000)
        await cmd("use comb on grate"); await drain()
        await shot("m2-05-alley-mime-fed")
        await click("Walk", 318, 140, 5000)        # east edge -> boardwalk
        await drain()
        await click("Walk", 152, 103, 6000)        # into the Lucky Gull
        await drain()
        # --- Store
        await shot("m2-06-store")
        await click("Talk", 60, 70, 7000); await drain()
        await cmd("buy carnation"); await drain()
        await cmd("buy mints"); await drain()
        await cmd("wear carnation"); await drain()
        await cmd("save"); await drain()
        await click("Use", 282, 70, 8000)          # the slushie machine
        await drain(); await page.wait_for_timeout(400)
        await shot("m2-07-death-slushie")
        await page.keyboard.press("r"); await page.wait_for_timeout(500)
        await drain()
        await shot("m2-08-restored-in-store")
        await click("Walk", 160, 173, 6000)        # out to the boardwalk
        await drain()
        await click("Walk", 261, 103, 6000)        # into the casino
        await drain()
        # --- Casino
        await shot("m2-09-casino")
        await click("Walk", 238, 103, 6000)        # Tiny blocks the elevator
        await drain()
        await click("Talk", 238, 95, 6000); await drain()
        await shot("m2-10-casino-approved")
        await click("Walk", 238, 103, 5000)        # ride up
        await drain()
        # --- Rooftop
        await shot("m2-11-rooftop")
        await cmd("eat mints"); await drain()
        await click("Talk", 106, 110, 5000)
        await drain(8); await page.wait_for_timeout(600)
        await shot("m2-12-ending")
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
        await shot("m2-13-legacy-save-restored")
        keys = await page.evaluate("[localStorage.getItem('neon-knights.save.v1'), localStorage.getItem('neon-nights.save.v1')]")
        print("after migration [new, old]:", keys)
        print("title:", await page.title())
        await b.close()
    print("console:", *logs, sep="\n  ")
    print("pageerrors:", errors)
    bad = [l for l in logs if l.startswith(("error", "warning"))] + errors
    sys.exit(1 if bad else 0)

asyncio.run(main())
