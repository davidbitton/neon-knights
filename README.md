# Neon Knights: A Gentleman's Evening Out

[![Rust](https://github.com/davidbitton/neon-knights/actions/workflows/rust.yml/badge.svg)](https://github.com/davidbitton/neon-knights/actions/workflows/rust.yml)
[![UI tests](https://github.com/davidbitton/neon-knights/actions/workflows/ui.yml/badge.svg)](https://github.com/davidbitton/neon-knights/actions/workflows/ui.yml)
[![Pages](https://github.com/davidbitton/neon-knights/actions/workflows/pages.yml/badge.svg)](https://github.com/davidbitton/neon-knights/actions/workflows/pages.yml)

**[Play it in your browser: https://davidbitton.github.io/neon-knights/](https://davidbitton.github.io/neon-knights/)**

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
* **Sound**: square-wave PC-speaker beeps (a title jingle, score blips, a
  death dirge, an ending fanfare, and a buzz when the parser is confused). Type
  `sound off` / `sound on` (or `mute` / `unmute`), or press **M** whenever
  you're not typing a command (title screen, message boxes, tombstone,
  ending). The setting is remembered in `localStorage`
  (`neon-knights.sound`). Audio starts only after your first key press or
  click, as browsers require.
* The game saves to one `localStorage` slot (`neon-knights.save.v1`). The save
  stores the scene, Chip's position, score, inventory, cash, every puzzle
  flag and conversation progress. It's a versioned text record (`NN2|...`).
  Milestone 1 saves (`NN1|...`, always in the bar) still load. So do saves
  made under the old working title "Neon Nights" (`neon-nights.save.v1`),
  which are migrated to the new key on restore.

There are **60 points**. Every point can be earned, and each one only once.
There are **five ways to die**, and each gets its own tombstone and epitaph.
Stuck? [WALKTHROUGH.md](WALKTHROUGH.md) has the full solution (spoilers!).

## Continuous integration and deployment

* `.github/workflows/rust.yml` ("Rust") is GitHub's Rust starter workflow,
  extended. On every push and pull request to `main` it runs on a pinned
  toolchain: `cargo fmt --check`, build, tests, clippy (native and
  `wasm32-unknown-unknown`, with `-D warnings`) and a release wasm32 build.
* `.github/workflows/ui.yml` ("UI tests") runs on every push and pull
  request to `main`. It builds the game with trunk, installs pinned
  Playwright + Chromium (cached), and runs the browser test suite below. If
  anything fails, it uploads the visual diffs and failure screenshots as the
  `ui-test-results` artifact.
* `.github/workflows/pages.yml` ("Pages") runs on every push to `main` (or
  by hand). It builds the site with
  `trunk build --release --public-url /neon-knights/` and deploys it to
  GitHub Pages. In the repository settings, Pages must be set to
  **Source: GitHub Actions**.

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

## Testing

**Native tests** (`cargo test`) cover the engine, the parser, every scene's
rules, a full walkthrough to 60/60, every death, save compatibility,
path-finding, the font and the tunes.

**Browser UI tests** (`tests/ui/`, pytest + Playwright, Chromium) drive the
real game in a headless browser:

* *Functional* tests (`test_game.py`) use real keyboard and mouse input:
  - a full mouse-and-keyboard playthrough to 60/60
  - every death, plus the tombstone's Restore button and N key
  - save/restore, including migration of the legacy save key
  - the ending's buttons and keys
  - audio: the AudioContext only after a user gesture, and the beeps for
    each sound effect
  - mute (M key and `sound on`/`sound off`) persisting across reloads
* *Visual regression* tests (`test_visual.py`) compare key frames: the
  title, all six scenes, a message box, a parser error, a death screen and
  the ending.

  They compare the game's own 320x200 framebuffer, read out as palette
  indices, against PNG baselines in `tests/visual/baseline/`. They don't use
  page screenshots, so fonts, GPU and OS rendering can't change the result.
  By default a single differing pixel fails the test
  (`VISUAL_MAX_DIFF_PIXELS=0`; `VISUAL_COLOR_TOLERANCE=8` per channel).
  On failure, `tests/visual/output/` gets `<name>-actual.png` and
  `<name>-diff.png` (baseline | actual | changes in red, ringed in yellow).

The tests are deterministic. They open the game with `?test`, which stops
the game clock, so time only advances when a test calls the
wasm-exported `test_step` / `test_until_idle` hooks. They read game state
(`test_state`) and pixels (`test_framebuffer`) through those hooks too.
There are no fixed sleeps. The stars and all text rotation in the game are
already seeded or deterministic. Any console error or warning, or any
uncaught page error, fails the test.

```sh
python3 -m venv .venv && . .venv/bin/activate
pip install -r tests/requirements.txt          # pinned versions
python -m playwright install --with-deps chromium

trunk build --release
python -m pytest tests/ui                      # serves dist/ on a free port

# Update the visual baselines on purpose after an intended art/text change,
# then review and commit tests/visual/baseline/*.png:
UPDATE_SNAPSHOTS=1 python -m pytest tests/ui -k visual
```

To run against another server, set `NK_BASE_URL`. For example, to check the
GitHub Pages build under its project subpath:

```sh
trunk build --release --public-url /neon-knights/ --dist /tmp/site/neon-knights
(cd /tmp/site && python3 -m http.server 8081 &)
NK_BASE_URL=http://localhost:8081/neon-knights/ python -m pytest tests/ui
```

## Layout

```
src/
  lib.rs            crate root; `web` is compiled only for wasm32
  engine/           platform-independent, unit-tested natively
    palette.rs      16-color EGA palette
    font.rs         "Chip Mono", an original 5x8 bitmap font drawn into the framebuffer
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
    sound.rs        PC-speaker tunes as (frequency, duration) note lists
    text.rs         word wrap and message pagination
  web.rs            wasm-bindgen/web-sys front-end: canvas, input, UI, Web Audio, localStorage
index.html          trunk entry point; nearest-neighbor scaled canvas
tests/ui/           browser UI tests (pytest + Playwright): functional + visual
tests/visual/baseline/  committed framebuffer baselines for the visual tests
.github/workflows/  CI (rust.yml), UI tests (ui.yml), GitHub Pages (pages.yml)
```

How rendering works: every frame the scene's *visual* plane is copied into an
indexed 320x200 frame, sprites are drawn only where their priority (derived from
their feet's y position, in 12-pixel bands) is at least the picture's priority
at that pixel, then the frame is expanded to RGBA and put on a 320x200 canvas
that CSS scales up with `image-rendering: pixelated`. All text is drawn into
the same framebuffer with the built-in bitmap font: the status bar, the command
line, message boxes, tombstones, the ending, and the title (drawn at 4x and 2x
scale). So text scales and crunches exactly like the art does.

## License

MIT. See [LICENSE](LICENSE).
