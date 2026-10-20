import asyncio, os, sys
from playwright.async_api import async_playwright

URL = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8765/"
OUT = os.environ.get("SHOTS_DIR", "/workspace/neon-nights-shots")

async def main():
    logs, errors = [], []
    async with async_playwright() as p:
        b = await p.chromium.launch()
        page = await b.new_page(viewport={"width": 1280, "height": 800})
        page.on("console", lambda m: logs.append(f"{m.type}: {m.text}"))
        page.on("pageerror", lambda e: errors.append(str(e)))
        await page.goto(URL)
        await page.wait_for_timeout(1500)
        box = await page.locator("#hi").bounding_box()
        async def click_fb(x, y, button="left"):
            await page.mouse.click(box["x"] + (x + 0.5) * box["width"] / 320,
                                   box["y"] + (y + 0.5) * box["height"] / 200, button=button)
        async def cmd(text):
            await page.keyboard.type(text, delay=15)
            await page.keyboard.press("Enter")
            await page.wait_for_timeout(300)
        async def ok():
            await page.keyboard.press("Enter"); await page.wait_for_timeout(250)
        async def shot(name):
            await page.locator("#screen").screenshot(path=f"{OUT}/{name}.png")
            print("shot", name)

        await shot("01-title")
        await page.keyboard.press("Enter"); await page.wait_for_timeout(400)
        await shot("02-intro")
        await ok()
        await cmd("look at the bartender")
        await shot("03-bar-look-bartender")
        await ok()
        await cmd("xyzzy")
        await shot("04-parser-snark")
        await ok()
        # Take verb via Tab x1 (Look -> Take), then click the matchbook: hero auto-walks.
        await page.keyboard.press("Tab")
        await click_fb(120, 79)
        await page.wait_for_timeout(9000)
        await shot("05-took-matchbook")
        await ok()
        await cmd("read matchbook"); await ok()
        await cmd("save"); await ok()
        # walk behind a stool (priority demo): click floor at stool 102, y 123
        await page.keyboard.press("Tab"); await page.keyboard.press("Tab"); await page.keyboard.press("Tab")  # -> Walk
        await click_fb(96, 124)
        await page.wait_for_timeout(5000)
        await shot("06-behind-stool")
        await cmd("eat egg")   # too far
        await ok()
        await page.keyboard.press("Tab"); await page.keyboard.press("Tab")  # Walk -> Look -> Take
        await click_fb(193, 74)  # take egg -> walks, Gus warns
        await page.wait_for_timeout(6000)
        await ok()
        await cmd("eat the pickled egg")
        await ok(); await ok()
        await page.wait_for_timeout(500)
        await shot("07-death")
        await page.keyboard.press("r"); await page.wait_for_timeout(500)
        await shot("08-restored")
        await ok()
        # Finish the scene from the restored save.
        for _ in range(2): await page.keyboard.press("Tab")   # Take -> Talk -> Use
        await click_fb(236, 85)    # use jukebox (walks over first)
        await page.wait_for_timeout(7000)
        await ok()
        await page.wait_for_timeout(600)
        await shot("09-jukebox-playing")
        for _ in range(4): await page.keyboard.press("Tab")   # Use -> Walk -> Look -> Take -> Talk
        await click_fb(151, 70)    # talk to bartender
        await page.wait_for_timeout(7000)
        await ok()
        await cmd("drink the special"); await ok()
        await cmd("score")
        await shot("10-score")
        await ok()
        await cmd("leave")
        await shot("11-leave")
        await ok()
        await cmd("quit")
        await shot("12-quit")
        state = await page.evaluate("localStorage.getItem('neon-knights.save.v1')")
        print("save:", state)
        # Legacy-key migration: a save left by a "Neon Nights" build must restore.
        await page.evaluate("""() => {
            localStorage.removeItem('neon-knights.save.v1');
            localStorage.setItem('neon-nights.save.v1', 'NN1|13|7|cmd|236|112|9|0|0');
        }""")
        await page.reload(); await page.wait_for_timeout(1500)
        await page.keyboard.press("Enter"); await page.wait_for_timeout(300)
        await ok()
        await cmd("restore")
        await shot("13-legacy-save-restored")
        keys = await page.evaluate("[localStorage.getItem('neon-knights.save.v1'), localStorage.getItem('neon-nights.save.v1')]")
        print("after migration [new, old]:", keys)
        title = await page.title()
        print("title:", title)
        await b.close()
    print("console:", *logs, sep="\n  ")
    print("pageerrors:", errors)

asyncio.run(main())
