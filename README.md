# Neon Nights: A Gentleman's Evening Out

A tiny browser adventure game written in plain Rust and compiled to
WebAssembly. Chip Valentine, a cheesy guy in a powder-blue tuxedo, spends one
night in Ocean Bluffs, a run-down beach town. Milestone 1 contains a single
fully playable scene: **The Rusty Anchor**, a dive bar.

> **Neon Nights is an original parody/homage** in the style of the late-1980s
> Sierra-style parser adventures (16-color EGA graphics, a typed command line,
> a status bar with your score, and comedic deaths). All characters, story,
> writing, art and code are original. **No Sierra assets, characters, names,
> story, dialogue, art, music or puzzles are used.** It isn't affiliated with
> or endorsed by anyone who made those games.

## Playing

* **Type commands** at the `>` prompt and press Enter: `look`, `look at bartender`,
  `take matchbook`, `read matchbook`, `talk to gus`, `use quarter on jukebox`,
  `order drink`, `inventory` / `i`, `score`, `help`, `save`, `restore`, `restart`, `quit`.
  Synonyms (`examine`, `x`, `l`, `get`, `grab`, `pick up` ...) and filler words
  (`the`, `at`, `to`, `with` ...) are understood.
* **Arrow keys** walk (press the same arrow again to stop, old-school).
* **Click** the floor to walk there. Click a thing to apply the current
  click-verb to it (it types the command for you, e.g. clicking Gus with the
  Look verb runs `look bartender`). **Right-click or Tab** cycles the verb
  (Walk / Look / Take / Talk / Use). Non-look verbs walk Chip over first.
* Message boxes pause the game: press Enter, Esc, Space or click to dismiss.
* The game saves to one `localStorage` slot.

There are 20 points in the Rusty Anchor, and at least one way to die.

## Building

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

Optional browser smoke test (Playwright, Chromium) that plays through the
scene and writes screenshots to `/workspace/neon-nights-shots/`:

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
  game/             pure game logic, no browser types
    parser.rs       verb-noun parser: synonyms, filler words, errors
    state.rs        score, inventory, flags, save-game (de)serialization
    logic.rs        command handling, scoring, death, and all of the writing
    scene.rs        vector pictures (bar, title, tombstone) + hotspot geometry
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
