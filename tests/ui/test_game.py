"""Functional browser tests: real key/mouse input, deterministic time."""
from __future__ import annotations

import pytest

from nk import SAVES, Game, Save


def test_hooks_need_test_mode(page, nk_url, console):
    page.goto(nk_url)
    page.wait_for_function("typeof window.wasmBindings?.test_state === 'function'")
    err = page.evaluate("() => { try { wasmBindings.test_state(); return null } catch (e) { return String(e) } }")
    assert err and "?test" in err


def test_title_then_intro(game: Game):
    st = game.state()
    assert st["mode"] == "Title" and st["ticks"] == 0
    msgs = game.start()
    assert msgs[0].startswith("Ocean Bluffs, 11:42 PM")
    st = game.state()
    assert (st["mode"], st["scene"], st["score"]) == ("Playing", "Bar", 0)
    assert st["hero"] == [40, 150]


def test_clock_is_frozen_until_stepped(game: Game):
    game.start()
    t0 = game.state()["ticks"]
    game.page.evaluate("() => new Promise(r => setTimeout(r, 300))")
    assert game.state()["ticks"] == t0
    game.step(7)
    assert game.state()["ticks"] == t0 + 7


def test_parser_snark_and_help_pages(game: Game):
    game.start()
    game.type("xyzzy")
    [msg] = game.messages()
    assert "xyzzy" in msg
    game.key("F1")
    pages = game.messages()
    assert len(pages) >= 2 and "LOOK" in pages[0]


def test_full_playthrough_by_mouse_and_keyboard(game: Game):
    """The whole walkthrough to 60/60, walking everywhere by clicking."""
    game.start()
    # The Rusty Anchor.
    assert game.act("Take", 120, 79)["score"] == 5
    game.messages()
    assert game.type("read matchbook")["score"] == 8
    game.messages()
    assert game.act("Use", 236, 85)["score"] == 13
    game.messages()
    assert game.act("Talk", 151, 70)["score"] == 18
    game.messages()
    assert game.type("drink the special")["score"] == 20
    game.messages()
    assert game.walk(278, 103)["scene"] == "Boardwalk"  # out the front door
    game.messages()
    # Boardwalk.
    game.act("Talk", 168, 108)
    game.messages()
    assert game.type("compliment dot")["score"] == 23
    game.messages()
    assert game.act("Use", 296, 130)["score"] == 25  # telescope
    game.messages()
    assert game.walk(2, 140)["scene"] == "Alley"  # west edge
    game.messages()
    # Alley.
    game.walk(226, 140)
    assert game.type("give hot dog to mime")["score"] == 30
    game.messages()
    game.walk(170, 158)
    st = game.type("use comb on grate")
    assert (st["score"], st["cash"]) == (35, 5)
    game.messages()
    assert game.walk(318, 140)["scene"] == "Boardwalk"  # east edge
    assert game.walk(152, 103)["scene"] == "Store"
    game.messages()
    # Store.
    game.act("Talk", 60, 70)
    game.messages()
    assert game.type("buy carnation")["score"] == 38
    game.messages()
    assert game.type("buy mints")["score"] == 41
    game.messages()
    assert game.type("wear carnation")["score"] == 43
    game.messages()
    game.type("save")
    game.messages()
    assert game.walk(160, 173)["scene"] == "Boardwalk"
    assert game.walk(261, 103)["scene"] == "Casino"
    game.messages()
    # Casino: Tiny blocks the elevator until asked.
    st = game.walk(238, 103)
    assert st["scene"] == "Casino"
    assert "Talk to me first" in game.messages()[0]
    assert game.act("Talk", 238, 95)["score"] == 48
    game.messages()
    assert game.walk(238, 103)["scene"] == "Rooftop"
    game.messages()
    # Rooftop.
    assert game.type("eat mints")["score"] == 50
    game.messages()
    game.act("Talk", 106, 110)
    msgs = game.messages()
    assert msgs[-1] == '"You\'re hired."'
    st = game.state()
    assert (st["mode"], st["score"]) == ("Ending", 60)
    # Ending offers Restore / Restart / Quit: Restore brings back the store save.
    game.click_fb(93, 187)
    assert game.messages()[0].startswith("Game restored")
    st = game.state()
    assert (st["mode"], st["scene"], st["score"]) == ("Playing", "Store", 43)


DEATHS = [
    # (name, save or None for a fresh bar, walk-to point, commands, cause)
    ("egg", None, (193, 118), ["eat egg", "eat egg"], "pickled egg"),
    ("dumpster", "alley", (118, 124), ["climb into dumpster"], "trash"),
    ("slushie", "store", (282, 113), ["drink slushie"], "Brain freeze"),
    ("jackpot", Save("Casino", 20, ["SeenIntro", "VisitedCasino"], "qc", 0, (160, 160)), (70, 122), ["play slots"], "jackpot"),
    ("pool", "rooftop", (178, 142), ["dive into pool"], "swan dive"),
]


@pytest.mark.parametrize("name,save,spot,cmds,cause", DEATHS, ids=[d[0] for d in DEATHS])
def test_every_death_and_its_buttons(game: Game, name, save, spot, cmds, cause):
    if save is None:
        game.start()
        game.type("save")
        game.messages()
    else:
        game.restore(SAVES[save] if isinstance(save, str) else save)
        game.type("save")
        game.messages()
    scene = game.state()["scene"]
    game.walk(*spot)
    for c in cmds:
        game.type(c)
        game.messages()
    st = game.state()
    assert st["mode"] == "Dead" and st["dead"], st
    # The cause is drawn in the framebuffer; check state-side via restore.
    game.click_fb(157, 160)  # Restore button
    assert game.messages()[0].startswith("Game restored")
    st = game.state()
    assert (st["mode"], st["scene"], st["dead"]) == ("Playing", scene, False)
    # And N on the tombstone restarts from scratch.
    game.walk(*spot)
    for c in cmds:
        game.type(c)
        game.messages()
    game.key("n")
    st = game.state()
    assert (st["mode"], st["scene"], st["score"]) == ("Playing", "Bar", 0)


def test_save_restore_and_legacy_key(game: Game):
    game.start()
    game.act("Take", 120, 79)
    game.messages()
    game.type("save")
    game.messages()
    assert game.storage("neon-knights.save.v1").startswith("NN2|0|5|")
    game.type("restart")
    game.messages()
    assert game.state()["score"] == 0
    game.type("restore")
    game.messages()
    assert game.state()["score"] == 5
    # An M1-era save under the old "Neon Nights" key is migrated.
    game.page.evaluate("""() => {
        localStorage.removeItem('neon-knights.save.v1');
        localStorage.setItem('neon-nights.save.v1', 'NN1|13|7|cmd|236|112|9|0|0');
    }""")
    game.type("restore")
    game.messages()
    st = game.state()
    assert (st["scene"], st["score"], st["inventory"]) == ("Bar", 13, "cmd")
    assert game.storage("neon-nights.save.v1") is None
    assert game.storage("neon-knights.save.v1").startswith("NN1|13|")


def test_audio_waits_for_a_gesture_and_mute_sticks(probed_game: Game):
    g = probed_game
    p = g.probe()
    assert p["created"] == [] and p["oscs"] == 0, "no audio before a user gesture"
    g.key("Enter")
    p = g.probe()
    assert p["created"] == [1], "AudioContext made inside the first gesture"
    assert p["oscs"] == 9, "title jingle: 9 notes"
    g.messages()
    g.type("xyzzy")
    assert g.probe()["oscs"] == 11, "parser buzz: 2 notes"
    g.messages()
    g.act("Take", 120, 79)
    assert g.probe()["oscs"] == 13, "score blip: 2 notes"
    g.messages()
    # Die: the dirge plays when the tombstone appears.
    g.walk(193, 118)
    g.type("eat egg")
    g.messages()
    g.type("eat egg")
    g.messages()
    assert g.state()["mode"] == "Dead"
    assert g.probe()["oscs"] == 13 + 8, "death dirge: 8 notes"
    # M mutes from the tombstone and is remembered across reloads.
    g.key("m")
    assert g.state()["muted"] and g.storage("neon-knights.sound") == "off"
    g.reload()
    assert g.state()["muted"]
    g.key("Enter")
    assert g.probe()["oscs"] == 0, "muted: no jingle"
    g.messages()
    g.type("frobnicate")
    g.messages()
    assert g.probe()["oscs"] == 0
    assert g.type("sound on")["muted"] is False
    assert g.messages()[0].startswith("Sound on")
    assert g.storage("neon-knights.sound") == "on"
    assert g.probe()["oscs"] == 2, "a confirmation blip"
    assert g.type("mute")["muted"] is True


def test_ending_fanfare_and_keys(probed_game: Game):
    g = probed_game
    g.restore(Save("Rooftop", 50, SAVES["rooftop"].flags + ["AteMints"], "cmf", 0, (124, 134)))
    before = g.probe()["oscs"]
    g.type("sing to vonda")
    g.messages()
    st = g.state()
    assert (st["mode"], st["score"]) == ("Ending", 60)
    assert g.probe()["oscs"] == before + 9, "ending fanfare: 9 notes (2 rests)"
    g.key("q")
    assert g.state()["mode"] == "Quit"
    g.key("Enter")
    assert g.state()["mode"] == "Title"
