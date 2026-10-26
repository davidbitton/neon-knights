"""Functional browser tests: real key/mouse input, deterministic time."""
from __future__ import annotations

import pytest

from nk import FLAG_BITS, SAVES, Game, Save, pelican_x


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
    """The whole walkthrough to 80/80, walking everywhere by clicking."""
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
    assert game.act("Talk", 106, 110)["score"] == 62  # the song, and a claim ticket
    assert "ticket" in " ".join(game.messages()).lower()
    st = game.state()
    assert (st["mode"], st["inventory"].count("t")) == ("Playing", 1)
    # The Second Night. Suds & Duds: Dee explains the pelican.
    game.travel("casino", "boardwalk", "alley")
    assert game.walk(2, 140)["scene"] == "Laundromat"
    game.messages()
    assert game.act("Talk", 137, 105)["score"] == 64
    game.messages()
    # The pier: borrow Mort's rod.
    game.walk(318, 140)
    game.messages()
    game.travel("boardwalk")
    assert game.walk(318, 128)["scene"] == "Pier"
    game.messages()
    assert game.act("Take", 95, 80)["score"] == 66
    game.messages()
    assert game.walk(318, 128)["scene"] == "Motel"
    game.messages()
    # The motel: comb Ferris out of the lobby, take the shrimp.
    game.walk(200, 126)
    game.type("use comb on ferris")
    game.messages()
    assert game.act("Take", 73, 90)["score"] == 68
    game.messages()
    assert game.walk(2, 128)["scene"] == "Pier"
    game.messages()
    # Fish for a flounder at the railing, trade it to Percival for the key.
    game.walk(216, 142)
    assert game.type("use rod")["score"] == 71
    game.messages()
    game.walk(pelican_x(game.state()["ticks"]), 134)
    assert game.type("give flounder to pelican")["score"] == 75
    game.messages()
    assert "k" in game.state()["inventory"]
    # Back to Dryer 3.
    game.travel("boardwalk", "alley", "laundromat")
    game.walk(199 + 64, 108)
    assert game.type("use key on dryer")["score"] == 78
    game.messages()
    assert "g" in game.state()["inventory"]
    # Return the gown to Vonda: the finale.
    game.travel("alley", "boardwalk", "casino")
    assert game.walk(238, 103)["scene"] == "Rooftop"
    game.messages()
    game.walk(124, 134)
    game.type("give gown to vonda")
    msgs = game.messages()
    assert msgs[-1].endswith("hired.\"") or "hired" in msgs[-1]
    st = game.state()
    assert (st["mode"], st["score"]) == ("Ending", 80)
    # Ending offers Restore / Restart / Quit: Restore brings back the store save.
    game.click_fb(93, 187)
    assert game.messages()[0].startswith("Game restored")
    st = game.state()
    assert (st["mode"], st["scene"], st["score"]) == ("Playing", "Store", 43)


DRYER_OPEN = Save("Laundromat", 78, [*SAVES["dryer"].flags, "DryerOpen", "GotGown"], "cmfrg", 0, (300, 140))
SHRIMP_AWAY = Save("Motel", 66, SAVES["motel"].flags + ["FerrisAway"], "cmfr", 0, (16, 128))

DEATHS = [
    # (name, save or None for a fresh bar, walk-to point, commands, cause)
    ("egg", None, (193, 118), ["eat egg", "eat egg"], "pickled egg"),
    ("dumpster", "alley", (118, 124), ["climb into dumpster"], "trash"),
    ("slushie", "store", (282, 113), ["drink slushie"], "Brain freeze"),
    ("jackpot", Save("Casino", 20, ["SeenIntro", "VisitedCasino"], "qc", 0, (160, 160)), (70, 122), ["play slots"], "jackpot"),
    ("pool", "rooftop", (178, 142), ["dive into pool"], "swan dive"),
    ("dryer", DRYER_OPEN, (263, 108), ["climb into dryer"], "Tumble dried"),
    ("shrimp", SHRIMP_AWAY, (74, 120), ["eat shrimp"], "shrimp"),
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
    assert game.storage("neon-knights.save.v1").startswith("NN3|0|5|")
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
    g.restore(SAVES["gown"])
    before = g.probe()["oscs"]
    g.type("give gown to vonda")
    g.messages()
    st = g.state()
    assert (st["mode"], st["score"]) == ("Ending", 80)
    assert g.probe()["oscs"] == before + 9, "ending fanfare: 9 notes (2 rests)"
    g.key("q")
    assert g.state()["mode"] == "Quit"
    g.key("Enter")
    assert g.state()["mode"] == "Title"


# --------------------------------------------------------------- Milestone 4


def test_the_map_grew_three_scenes(game: Game):
    game.restore(SAVES["boardwalk"])
    # Boardwalk -> Pier -> Motel and back.
    assert game.walk(318, 128)["scene"] == "Pier"
    assert "Gull Point" in game.messages()[0]
    assert game.walk(318, 128)["scene"] == "Motel"
    assert "Sandpiper" in game.messages()[0]
    assert game.walk(2, 128)["scene"] == "Pier"
    game.messages()
    assert game.walk(2, 128)["scene"] == "Boardwalk"
    game.messages()
    # Alley <-> Laundromat.
    game.travel("alley")
    assert game.walk(2, 140)["scene"] == "Laundromat"
    assert "SUDS & DUDS" in game.messages()[0]
    assert game.walk(318, 140)["scene"] == "Alley"
    game.messages()
    st = game.state()
    for flag in ("VisitedPier", "VisitedMotel", "VisitedLaundromat"):
        assert st["flags"] >> FLAG_BITS[flag] & 1, flag


def test_dee_comes_first_then_the_key_works(game: Game):
    """The key won't open Dryer 3 until Dee has said she'll look away."""
    blind = Save("Laundromat", 71, [f for f in SAVES["dryer"].flags if f != "DeeBriefed"], "cmfrk", 0, (300, 140))
    game.restore(blind)
    game.walk(263, 108)
    st = game.type("use key on dryer")
    assert st["score"] == 71 and "g" not in st["inventory"]
    assert "ticket" in " ".join(game.messages()).lower()
    # Dee wants the claim ticket before she looks the other way.
    game.restore(SAVES["dryer"])
    game.walk(263, 108)
    assert game.type("use key on dryer")["score"] == 78
    msgs = game.messages()
    assert "sequins" in " ".join(msgs)
    assert "g" in game.state()["inventory"]
    # Points are paid once.
    assert game.type("use key on dryer")["score"] == 78
    game.messages()


def test_pier_rod_bait_and_pelican(game: Game):
    game.restore(SAVES["pier"])
    # The rod is already in the pack; without bait the sea just laughs.
    game.walk(216, 142)
    st = game.type("use rod")
    assert st["score"] == 66 and "b" not in st["inventory"]
    assert "bait" in " ".join(game.messages()).lower()
    # The pelican won't take shrimp for the key, only a flounder.
    game.restore(Save("Pier", 68, SAVES["pier"].flags + ["GotShrimp"], "cmfrs", 0, (16, 128)))
    game.walk(216, 142)
    assert game.type("use rod")["score"] == 71
    game.messages()
    st = game.state()
    assert "b" in st["inventory"] and "s" not in st["inventory"]
    game.walk(pelican_x(st["ticks"]), 134)
    assert game.type("give flounder to pelican")["score"] == 75
    assert "k" in game.state()["inventory"]
    game.messages()


def test_percival_roams_the_pier(game: Game):
    game.restore(SAVES["pier"])
    game.step(3)
    a = game.frame().crop((100, 100, 260, 150))
    game.step(120)
    b = game.frame().crop((100, 100, 260, 150))
    assert a.tobytes() != b.tobytes(), "the pelican should have moved"
    # His hotspot follows him: talk to him wherever he has got to.
    game.walk(pelican_x(game.state()["ticks"]), 134)
    st = game.type("talk to pelican")
    assert st["mode"] == "Playing"
    assert "Percival" in " ".join(game.messages())


def test_motel_ferris_the_comb_and_the_shrimp(game: Game):
    game.restore(SAVES["motel"])
    # Ferris guards the buffet: "Guests only."
    game.walk(74, 120)
    st = game.type("take shrimp")
    assert st["score"] == 66 and "s" not in st["inventory"]
    assert "guests" in " ".join(game.messages()).lower()
    # Comb her hair; she leaves the lobby and takes her hotspot with her.
    game.walk(200, 126)
    game.type("use comb on ferris")
    game.messages()
    st = game.state()
    assert st["flags"] >> FLAG_BITS["FerrisAway"] & 1 and "c" in st["inventory"]
    game.walk(74, 120)
    assert game.type("take shrimp")["score"] == 68
    game.messages()
    assert "s" in game.state()["inventory"]


def test_gown_must_go_back_to_vonda(game: Game):
    """Singing alone no longer ends the game; the gown does."""
    game.restore(SAVES["sang"])
    st = game.type("sing to vonda")
    assert (st["mode"], st["score"]) == ("Playing", 62)
    game.messages()
    game.restore(SAVES["gown"])
    st = game.type("give gown to vonda")
    game.messages()
    st = game.state()
    assert (st["mode"], st["score"]) == ("Ending", 80)


def test_hint_command_is_progressive_and_saved(game: Game):
    game.start()
    seen = []
    for _ in range(4):
        game.type("hint")
        [m] = game.messages()
        seen.append(m)
    assert [m[:11] for m in seen] == ["Hint (1/3):", "Hint (2/3):", "Hint (3/3):", "Hint (3/3):"]
    assert seen[0] != seen[1] != seen[2]
    # The depth is saved: a restored game continues at the same level.
    game.type("save")
    game.messages()
    rec = game.storage("neon-knights.save.v1")
    assert rec.startswith("NN3|") and rec.split("|")[-1] != "0" * 24
    game.type("restart")
    game.messages()
    game.type("hint")
    assert game.messages()[0].startswith("Hint (1/3)")
    game.type("restore")
    game.messages()
    game.type("hint")
    assert game.messages()[0].startswith("Hint (3/3)")


def test_hint_points_onward_when_an_area_is_done(game: Game):
    game.restore(SAVES["dryer"])
    game.type("hint")
    [m] = game.messages()
    assert "Dryer" in m or "dryer" in m
    game.restore(SAVES["gown"])
    game.type("hint")
    assert game.messages()[0].startswith("Hint (1/3)")


def test_verbs_command_and_help_mention_it(game: Game):
    game.start()
    game.type("verbs")
    text = " ".join(game.messages())
    for word in ("LOOK", "GIVE", "SING", "HINT", "VERBS"):
        assert word in text
    game.key("F1")
    assert "HINT" in " ".join(game.messages())


def test_nn3_save_round_trip_and_old_saves_load(game: Game):
    game.restore(SAVES["gown"])
    game.type("save")
    game.messages()
    rec = game.storage("neon-knights.save.v1")
    head = rec.split("|")
    assert head[0] == "NN3" and len(head) == 12 and len(head[-1]) == 24
    game.type("restart")
    game.messages()
    game.type("restore")
    game.messages()
    st = game.state()
    assert (st["scene"], st["score"], st["inventory"]) == ("Rooftop", 78, "cmfrkg")
    # A Milestone 3 (NN2) record still loads, and re-saves as NN3.
    game.page.evaluate("d => localStorage.setItem('neon-knights.save.v1', d)", SAVES["rooftop"].encode_nn2())
    game.type("restore")
    assert game.messages()[0].startswith("Game restored")
    st = game.state()
    assert (st["scene"], st["score"]) == ("Rooftop", 48)
    game.type("save")
    game.messages()
    assert game.storage("neon-knights.save.v1").startswith("NN3|5|48|")
    # So does an NN1 record (already covered above), and garbage is refused.
    game.page.evaluate("() => localStorage.setItem('neon-knights.save.v1', 'NN9|nonsense')")
    game.type("restore")
    assert "Game restored" not in game.messages()[0]
    assert game.state()["score"] == 48


def test_new_items_have_descriptions(game: Game):
    game.restore(SAVES["gown"])
    for word, hint in [("rod", "rod"), ("key", "key"), ("gown", "gown")]:
        game.type(f"look at {word}")
        assert hint in " ".join(game.messages()).lower()
