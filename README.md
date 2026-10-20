# Neon Knights: A Gentleman's Evening Out

Repository: https://github.com/davidbitton/neon-knights

A tiny browser adventure game written in plain Rust and compiled to
WebAssembly. Chip Valentine, a cheesy guy in a powder-blue tuxedo, spends one
night in Ocean Bluffs, a run-down beach town, trying to reach the mysterious
Vonda on the casino roof. The game has six scenes:

| Scene | Who's there |
|---|---|
| **The Rusty Anchor**, a dive bar | Gus, the one-eyed bartender |
| **The Boardwalk**, the hub that connects everything | Dot, a roller-skating hot dog vendor |
| **Back Alley** | a mime trapped in an invisible box |
| **Lucky Gull 24/7**, a convenience store | Brenda, a deeply bored clerk |
| **The Golden Clam**, a casino lobby | Tiny, a doorman who is not tiny |
| **The Crow's Nest**, a rooftop lounge | Vonda, a lounge singer in sequins |

You get from scene to scene by walking through doorways or off the edge of
the screen, or by typing things like `enter store` or `go to casino`.

> **Neon Knights is an original parody/homage** in the style of the late-1980s
> Sierra-style parser adventures (16-color EGA graphics, a typed command line,
> a status bar with your score, and comedic deaths). All characters, story,
> writing, art and code are original. **No Sierra assets, characters, names,
> story, dialogue, art, music or puzzles are used.** It isn't affiliated with
> or endorsed by anyone who made those games.

## Playing

* **Type commands** at the `>` prompt and press Enter: `look`, `look at bartender`,
  `take matchbook`, `talk to gus`, `use quarter on jukebox`, `give hot dog to mime`,
  `buy carnation`, `wear carnation`, `enter casino`, `inventory` / `i`, `score`,
  `help`, `save`, `restore`, `restart`, `quit`.
  Synonyms (`examine`, `x`, `l`, `get`, `grab`, `pick up` ...) and filler words
  (`the`, `at`, `to`, `with` ...) are understood.
* **Arrow keys** walk (press the same arrow again to stop, old-school).
* **Click** the floor to walk there. Chip finds a route around the
  furniture, and he won't wander through a doorway unless you clicked it. Click a thing to apply the current
  click-verb to it (it types the command for you, e.g. clicking Gus with the
  Look verb runs `look bartender`). **Right-click or Tab** cycles the verb
  (Walk / Look / Take / Talk / Use). Non-look verbs walk Chip over first.
* Message boxes pause the game: press Enter, Esc, Space or click to dismiss.
* The game saves to one `localStorage` slot (`neon-knights.save.v1`). The save
  stores the scene, Chip's position, score, inventory, cash, every puzzle
  flag and conversation progress. It's a versioned text record (`NN2|...`).
  Milestone 1 saves (`NN1|...`, always in the bar) still load. So do saves
  made under the old working title "Neon Nights" (`neon-nights.save.v1`),
  which are migrated to the new key on restore.

There are **60 points**. Every point can be earned, and each one only once.
There are **five ways to die**, and each gets its own tombstone and epitaph.
Stuck? [WALKTHROUGH.md](WALKTHROUGH.md) has the full solution (spoilers!).

## Building

```sh
git clone https://github.com/davidbitton/neon-knights.git
cd neon-knights
```

Requirements: Rust (stable, via rustup), the `wasm32-unknown-unknown` target,
and [trunk](https://trunkrs.dev). Trunk downloads matching `wasm-bindgen` and
`wasm-opt` binaries automatically.

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk        # or download a prebuilt binary from its releases page

cargo test                 # native unit tests for engine + game logic
trunk serve --open         # dev server with live reload, or:
trunk build --release      # production build into dist/
cd dist && python3 -m http.server 8080   # then open http://localhost:8080
```

There's an optional browser smoke test (Playwright, Chromium). It plays the
whole game from the title screen to the ending, dies once along the way, and
writes screenshots (default `/workspace/neon-nights-shots/`, override
with `SHOTS_DIR`):

```sh
pip install playwright && playwright install chromium
python3 tools/smoke_test.py http://localhost:8080/
```

## Layout

```
src/
  lib.rs            crate root; `web` is compiled only for wasm32
  engine/           platform-independent, unit-tested natively
    palette.rs      16-color EGA palette
    gfx.rs          320x200 framebuffer; AGI-style visual / priority / control
                    planes; lines, rects, scanline polygons, ellipses, flood fill
    sprite.rs       ASCII-art sprites defined in code, priority-aware blitting,
                    hero walk cycle (4 directions), Gus the bartender
    actor.rs        movement over the walkable (control) plane, arrows + click-to-walk
    path.rs         breadth-first click-to-walk path-finding on the step lattice
  game/             pure game logic, no browser types
    parser.rs       verb-noun parser: synonyms, filler words, errors
    state.rs        scenes, score, inventory, flags, versioned save format
    logic/          command handling: shared rules (travel, items, deaths) in
                    mod.rs, one file of rules + writing per scene, and the
                    full-walkthrough / death / save tests
    world/          per-scene vector pictures, hotspots, exits and NPC
                    animation; npcs.rs (NPC sprites); screens.rs (title,
                    tombstones, ending)
    text.rs         word wrap
  web.rs            wasm-bindgen/web-sys front-end: canvas, input, UI, localStorage
index.html          trunk entry point; nearest-neighbor scaled canvas
tools/smoke_test.py headless-browser smoke test
```

How rendering works: every frame the scene's *visual* plane is copied into an
indexed 320x200 frame, sprites are drawn only where their priority (derived from
their feet's y position, in 12-pixel bands) is at least the picture's priority
at that pixel, then the frame is expanded to RGBA and put on a 320x200 canvas
that CSS scales up with `image-rendering: pixelated`. Text (status bar, command
line, message-box text) is drawn on a second, higher-resolution overlay canvas
on a fixed character grid; the boxes and borders themselves are framebuffer
pixels.

## License

MIT. See [LICENSE](LICENSE).
