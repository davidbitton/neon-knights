//! Browser front-end: canvas output, keyboard/mouse input, Web Audio beeps,
//! localStorage. All game rules live in `crate::game`; this file only wires
//! them to the DOM. Everything, text included, is drawn into the 320x200
//! framebuffer, which one CSS-scaled canvas displays.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{
    AudioContext, AudioContextState, CanvasRenderingContext2d, HtmlCanvasElement, ImageData,
    KeyboardEvent, MouseEvent, OscillatorType, Storage, Window,
};

use crate::engine::actor::Actor;
use crate::engine::font::{self, CELL_H as LH, CELL_W as CW};
use crate::engine::gfx::{priority_for_y, Frame, Picture, HEIGHT, WIDTH};
use crate::engine::palette::*;
use crate::engine::sprite::{hero_frame, Dir};
use crate::game::logic::{Death, Effect, Reply, HELP, INTRO, SOUND_OFF, SOUND_ON};
use crate::game::sound::{self, Sfx};
use crate::game::state::{GameState, SceneId, START_POS};
use crate::game::text::{paginate, wrap};
use crate::game::world::{self, npcs, screens, Placed, PIC_BOTTOM, STATUS_H};
use crate::game::{Game, LEGACY_SAVE_KEYS, SAVE_KEY, SUBTITLE, TITLE};

const TICK_MS: f64 = 50.0;
const WRAP: usize = 38;
/// Longest message page that fits in the picture area.
const MAX_LINES: usize = 15;
const MAX_INPUT: usize = 44;
const SOUND_KEY: &str = "neon-knights.sound";
const W: i32 = WIDTH as i32;

thread_local! {
    /// Set by a `?test` query parameter: the clock stops (time only moves
    /// via `test_step`) and the `test_*` hooks below become available.
    static TEST_MODE: Cell<bool> = const { Cell::new(false) };
    static APP: RefCell<Option<Rc<RefCell<App>>>> = const { RefCell::new(None) };
}

fn test_mode() -> bool {
    TEST_MODE.with(Cell::get)
}
const H: i32 = HEIGHT as i32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Title,
    Playing,
    Dead,
    Quit,
    Ending,
}

/// Click verbs (right-click / Tab cycles them).
const VERBS: [(&str, &str); 5] = [
    ("Walk", ""),
    ("Look", "look"),
    ("Take", "take"),
    ("Talk", "talk to"),
    ("Use", "use"),
];

/// Restore / Restart / Quit buttons: (label, x0, x1), plus their y span.
type Buttons = [(&'static str, i32, i32); 3];
const DEATH_BUTTONS: Buttons = [
    ("Restore", 126, 188),
    ("Restart", 190, 252),
    ("Quit", 254, 316),
];
const DEATH_BUTTON_Y: (i32, i32) = (152, 170);
const END_BUTTONS: Buttons = [
    ("Restore", 62, 124),
    ("Restart", 129, 191),
    ("Quit", 196, 258),
];
const END_BUTTON_Y: (i32, i32) = (179, 197);

#[derive(Clone, Copy)]
enum Choice {
    Restore,
    Restart,
    Quit,
}

fn button_at(buttons: &Buttons, ys: (i32, i32), x: i32, y: i32) -> Option<Choice> {
    if !(ys.0..ys.1).contains(&y) {
        return None;
    }
    let i = buttons
        .iter()
        .position(|(_, x0, x1)| (*x0..*x1).contains(&x))?;
    Some([Choice::Restore, Choice::Restart, Choice::Quit][i])
}

fn window() -> Window {
    web_sys::window().expect("no window")
}

fn storage() -> Option<Storage> {
    window().local_storage().ok().flatten()
}

/// Read the save slot, migrating a save left under a legacy key (from when
/// the game was called "Neon Nights") to the current key.
fn load_save() -> Option<String> {
    let s = storage()?;
    if let Some(d) = s.get_item(SAVE_KEY).ok().flatten() {
        return Some(d);
    }
    for old in LEGACY_SAVE_KEYS {
        if let Some(d) = s.get_item(old).ok().flatten() {
            if s.set_item(SAVE_KEY, &d).is_ok() {
                let _ = s.remove_item(old);
            }
            return Some(d);
        }
    }
    None
}

// -------------------------------------------------------------------- audio

/// A one-voice square-wave "PC speaker". The AudioContext is only created
/// inside a user gesture (keydown/mousedown), per browser autoplay rules.
struct Speaker {
    ctx: Option<AudioContext>,
    muted: bool,
}

const VOLUME: f32 = 0.06;

impl Speaker {
    fn new() -> Speaker {
        let muted = storage()
            .and_then(|s| s.get_item(SOUND_KEY).ok().flatten())
            .is_some_and(|v| v == "off");
        Speaker { ctx: None, muted }
    }

    /// Call from inside a user-gesture handler.
    fn unlock(&mut self) {
        if self.ctx.is_none() {
            self.ctx = AudioContext::new().ok();
        }
        if let Some(c) = &self.ctx {
            if c.state() == AudioContextState::Suspended {
                let _ = c.resume();
            }
        }
    }

    fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if let Some(s) = storage() {
            let _ = s.set_item(SOUND_KEY, if muted { "off" } else { "on" });
        }
    }

    fn play(&self, sfx: Sfx) {
        if self.muted {
            return;
        }
        let Some(ctx) = &self.ctx else { return };
        let mut t = ctx.current_time() + 0.02;
        for &(freq, ms) in sound::notes(sfx) {
            let d = ms as f64 / 1000.0;
            if freq > 0.0 {
                if let Err(e) = Self::tone(ctx, freq, t, d) {
                    web_sys::console::warn_1(&e);
                    return;
                }
            }
            t += d;
        }
    }

    fn tone(ctx: &AudioContext, freq: f32, t: f64, d: f64) -> Result<(), JsValue> {
        let osc = ctx.create_oscillator()?;
        osc.set_type(OscillatorType::Square);
        osc.frequency().set_value_at_time(freq, t)?;
        let gain = ctx.create_gain()?;
        gain.gain().set_value_at_time(VOLUME, t)?;
        // A hair of silence at the end keeps repeated notes distinct.
        gain.gain().set_value_at_time(0.0, t + d * 0.9)?;
        osc.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&ctx.destination())?;
        osc.start_with_when(t)?;
        osc.stop_with_when(t + d)?;
        Ok(())
    }
}

// -------------------------------------------------------------------- app

struct App {
    mode: Mode,
    game: Game,
    hero: Actor,
    /// The current scene's picture, cached by (scene, flags).
    pic: Picture,
    pic_key: (SceneId, u64),
    title_pic: Picture,
    tomb_pic: Picture,
    ending_pic: Picture,
    frame: Frame,
    rgba: Vec<u8>,
    input: String,
    last_cmd: Option<String>,
    queue: VecDeque<String>,
    death: Option<Death>,
    death_pending: bool,
    ending_pending: bool,
    verb: usize,
    pending: Option<String>,
    ticks: u64,
    acc: f64,
    last_t: Option<f64>,
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    speaker: Speaker,
}

impl App {
    fn new(ctx: CanvasRenderingContext2d, canvas: HtmlCanvasElement) -> App {
        let game = Game::new();
        let pic = world::draw(&game.state);
        let pic_key = (game.state.scene, game.state.flags);
        App {
            mode: Mode::Title,
            hero: Actor::new(START_POS.0, START_POS.1),
            game,
            pic,
            pic_key,
            title_pic: screens::draw_title(),
            tomb_pic: screens::draw_tomb(crate::game::DeathKind::Egg),
            ending_pic: screens::draw_ending(),
            frame: Frame::new(),
            rgba: Vec::with_capacity(WIDTH * HEIGHT * 4),
            input: String::new(),
            last_cmd: None,
            queue: VecDeque::new(),
            death: None,
            death_pending: false,
            ending_pending: false,
            verb: 1,
            pending: None,
            ticks: 0,
            acc: 0.0,
            last_t: None,
            ctx,
            canvas,
            speaker: Speaker::new(),
        }
    }

    // ---------------------------------------------------------------- flow

    fn say(&mut self, text: &str) {
        self.queue.extend(paginate(text, WRAP, MAX_LINES));
    }

    fn start_game(&mut self) {
        self.speaker.play(Sfx::Title);
        self.restart();
    }

    fn show_title(&mut self) {
        self.mode = Mode::Title;
        self.speaker.play(Sfx::Title);
    }

    fn restart(&mut self) {
        self.game = Game::new();
        self.hero = Actor::new(START_POS.0, START_POS.1);
        self.mode = Mode::Playing;
        self.queue.clear();
        self.say(INTRO);
        self.death = None;
        self.death_pending = false;
        self.ending_pending = false;
        self.pending = None;
        self.input.clear();
        self.last_cmd = None;
        self.sync_pic();
    }

    /// Redraw the scene picture if the scene or any flag changed.
    fn sync_pic(&mut self) {
        let key = (self.game.state.scene, self.game.state.flags);
        if key != self.pic_key {
            self.pic = world::draw(&self.game.state);
            self.pic_key = key;
        }
    }

    fn save(&mut self) {
        self.game.state.hero = (self.hero.x, self.hero.y);
        let data = self.game.state.serialize();
        let ok = storage()
            .map(|s| s.set_item(SAVE_KEY, &data).is_ok())
            .unwrap_or(false);
        self.say(if ok {
            "Game saved. Your progress is safe, which is more than can be said for your liver."
        } else {
            "Couldn't save: this browser won't let us use localStorage. Chip will have to rely on his memory. Uh-oh."
        });
    }

    fn restore(&mut self) {
        let data = load_save();
        match data.map(|d| GameState::deserialize(&d)) {
            Some(Ok(st)) => {
                self.hero = Actor::new(st.hero.0, st.hero.1);
                self.game.state = st;
                self.mode = Mode::Playing;
                self.death = None;
                self.death_pending = false;
                self.ending_pending = false;
                self.pending = None;
                self.last_cmd = None;
                self.input.clear();
                self.queue.clear();
                self.sync_pic();
                self.say(&format!("Game restored. Previously, on {TITLE}..."));
            }
            Some(Err(_)) => self.say("That saved game is corrupted. Much like this town."),
            None => self.say("There's no saved game. Chip lives in the moment, mostly because he forgets things."),
        }
    }

    fn choose(&mut self, c: Choice) {
        match c {
            Choice::Restore => self.restore(),
            Choice::Restart => self.restart(),
            Choice::Quit => {
                self.queue.clear();
                self.mode = Mode::Quit;
            }
        }
    }

    fn toggle_sound(&mut self, on: Option<bool>) -> &'static str {
        let on = on.unwrap_or(self.speaker.muted);
        self.speaker.set_muted(!on);
        if on {
            self.speaker.play(Sfx::Score);
            SOUND_ON
        } else {
            SOUND_OFF
        }
    }

    fn submit(&mut self, line: &str) {
        let line = line.trim().to_string();
        self.game.state.hero = (self.hero.x, self.hero.y);
        let reply = self.game.input(&line);
        self.last_cmd = Some(line);
        self.apply(reply);
    }

    fn apply(&mut self, reply: Reply) {
        for m in &reply.messages {
            self.say(m);
        }
        match reply.effect {
            Effect::None => {}
            Effect::Scored(_) => self.speaker.play(Sfx::Score),
            Effect::Confused => self.speaker.play(Sfx::Buzz),
            Effect::Sound(on) => {
                let msg = self.toggle_sound(on);
                self.say(msg);
            }
            Effect::Moved => {
                let (x, y) = self.game.state.hero;
                self.hero = Actor::new(x, y);
                self.pending = None;
                self.sync_pic();
            }
            Effect::Won => {
                self.hero.stop();
                self.pending = None;
                self.ending_pending = true;
                self.check_pending();
            }
            Effect::Died(d) => {
                self.hero.stop();
                self.pending = None;
                self.tomb_pic = screens::draw_tomb(d.kind);
                self.death = Some(d);
                self.death_pending = true;
                self.check_pending();
            }
            Effect::Save => self.save(),
            Effect::Restore => self.restore(),
            Effect::Restart => self.restart(),
            Effect::Quit => self.choose(Choice::Quit),
        }
    }

    fn dismiss(&mut self) {
        self.queue.pop_front();
        self.check_pending();
    }

    /// Once the last message is read, show the tombstone or the ending.
    fn check_pending(&mut self) {
        if !self.queue.is_empty() {
            return;
        }
        if self.death_pending {
            self.death_pending = false;
            self.mode = Mode::Dead;
            self.speaker.play(Sfx::Death);
        }
        if self.ending_pending {
            self.ending_pending = false;
            self.mode = Mode::Ending;
            self.speaker.play(Sfx::Ending);
        }
    }

    // ---------------------------------------------------------------- input

    fn on_key(&mut self, e: &KeyboardEvent) {
        if e.ctrl_key() || e.meta_key() || e.alt_key() {
            return;
        }
        self.speaker.unlock();
        let key = e.key();
        let lower = key.to_ascii_lowercase();
        let typing = self.mode == Mode::Playing && self.queue.is_empty();
        if lower == "m" && !typing {
            let on = self.speaker.muted;
            let msg = self.toggle_sound(Some(on));
            if self.mode == Mode::Playing {
                self.say(msg);
            }
            e.prevent_default();
            return;
        }
        let mut handled = true;
        match self.mode {
            Mode::Title => {
                if key == "Enter" || key == " " {
                    self.start_game();
                } else {
                    handled = false;
                }
            }
            Mode::Quit => {
                if key == "Enter" || key == " " {
                    self.show_title();
                } else {
                    handled = false;
                }
            }
            Mode::Dead | Mode::Ending => {
                if !self.queue.is_empty() {
                    if matches!(key.as_str(), "Enter" | "Escape" | " ") {
                        self.dismiss();
                    }
                } else {
                    match lower.as_str() {
                        "r" => self.choose(Choice::Restore),
                        "n" | "enter" => self.choose(Choice::Restart),
                        "q" => self.choose(Choice::Quit),
                        _ => handled = false,
                    }
                }
            }
            Mode::Playing => {
                if !self.queue.is_empty() {
                    if matches!(key.as_str(), "Enter" | "Escape" | " ") {
                        self.dismiss();
                    }
                } else {
                    match key.as_str() {
                        "ArrowUp" => self.arrow(Dir::Up),
                        "ArrowDown" => self.arrow(Dir::Down),
                        "ArrowLeft" => self.arrow(Dir::Left),
                        "ArrowRight" => self.arrow(Dir::Right),
                        "Enter" => {
                            let line = std::mem::take(&mut self.input);
                            self.submit(&line);
                        }
                        "Backspace" => {
                            self.input.pop();
                        }
                        "Escape" => self.input.clear(),
                        "Tab" => self.verb = (self.verb + 1) % VERBS.len(),
                        "F1" => self.say(HELP),
                        k if k.chars().count() == 1 => {
                            if self.input.chars().count() < MAX_INPUT {
                                self.input.push_str(k);
                            }
                        }
                        _ => handled = false,
                    }
                }
            }
        }
        if handled {
            e.prevent_default();
        }
    }

    /// Click-to-walk: path-find around furniture and away from doorways
    /// that aren't the destination.
    fn walk(&mut self, to: (i32, i32)) {
        let from = (self.hero.x, self.hero.y);
        let steps = world::plan_walk(self.game.state.scene, &self.pic, from, to);
        if steps.is_empty() {
            self.hero.walk_to(to.0, to.1);
        } else {
            self.hero.follow(steps);
        }
    }

    fn arrow(&mut self, d: Dir) {
        self.pending = None;
        self.hero.press_dir(d);
    }

    fn to_fb(&self, e: &MouseEvent) -> (i32, i32) {
        let r = self.canvas.get_bounding_client_rect();
        let x = ((e.client_x() as f64 - r.left()) / r.width() * WIDTH as f64).floor() as i32;
        let y = ((e.client_y() as f64 - r.top()) / r.height() * HEIGHT as f64).floor() as i32;
        (x, y)
    }

    fn on_mouse(&mut self, e: &MouseEvent) {
        self.speaker.unlock();
        let (x, y) = self.to_fb(e);
        let right = e.button() == 2;
        match self.mode {
            Mode::Title => self.start_game(),
            Mode::Quit => self.show_title(),
            Mode::Dead | Mode::Ending => {
                if !self.queue.is_empty() {
                    self.dismiss();
                    return;
                }
                let hit = if self.mode == Mode::Dead {
                    button_at(&DEATH_BUTTONS, DEATH_BUTTON_Y, x, y)
                } else {
                    button_at(&END_BUTTONS, END_BUTTON_Y, x, y)
                };
                if let Some(c) = hit {
                    self.choose(c);
                }
            }
            Mode::Playing => {
                if !self.queue.is_empty() {
                    self.dismiss();
                    return;
                }
                if right {
                    self.verb = (self.verb + 1) % VERBS.len();
                    return;
                }
                if !(STATUS_H..PIC_BOTTOM).contains(&y) {
                    return;
                }
                self.game.state.hero = (self.hero.x, self.hero.y);
                let hot = if self.verb == 0 {
                    None
                } else {
                    world::hotspot_at(&self.game.state, x, y)
                };
                match hot {
                    Some(h) => {
                        let (_, word) = VERBS[self.verb];
                        let cmd = format!("{word} {}", world::noun_word(h.noun));
                        if self.verb == 1 || world::is_near(&self.game.state, h.noun) {
                            self.submit(&cmd);
                        } else {
                            self.walk(h.approach);
                            self.pending = Some(cmd);
                        }
                    }
                    None => {
                        self.pending = None;
                        self.walk((x, y));
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------- update

    fn tick(&mut self) {
        self.ticks += 1;
        if self.mode != Mode::Playing || !self.queue.is_empty() {
            return;
        }
        self.sync_pic();
        let moved = self.hero.tick(&self.pic);
        self.game.state.hero = (self.hero.x, self.hero.y);
        if moved {
            if let Some(r) = self.game.check_exits() {
                self.apply(r);
                return;
            }
        }
        if !self.hero.is_moving() {
            if let Some(cmd) = self.pending.take() {
                self.submit(&cmd);
            }
        }
    }

    fn animate(&mut self, t: f64) {
        let last = self.last_t.unwrap_or(t);
        self.last_t = Some(t);
        self.acc = (self.acc + (t - last)).min(500.0);
        if test_mode() {
            // Frozen clock: tests advance time explicitly with test_step.
            self.acc = 0.0;
        }
        while self.acc >= TICK_MS {
            self.acc -= TICK_MS;
            self.tick();
        }
        if let Err(e) = self.render() {
            web_sys::console::error_1(&e);
        }
    }

    /// Tick until the hero has arrived and any queued click action has run,
    /// or a message box / mode change interrupts. Returns ticks used.
    fn until_idle(&mut self, max: u32) -> u32 {
        let mut n = 0;
        while n < max {
            if self.mode != Mode::Playing || !self.queue.is_empty() {
                break;
            }
            if !self.hero.is_moving() && self.pending.is_none() {
                break;
            }
            self.tick();
            n += 1;
        }
        n
    }

    fn state_json(&self) -> String {
        let st = &self.game.state;
        let msg = self
            .queue
            .front()
            .map_or("null".to_string(), |m| json_str(m));
        let inv: String = st.inventory.iter().map(|i| i.code()).collect();
        format!(
            concat!(
                "{{\"mode\":\"{:?}\",\"scene\":\"{:?}\",\"score\":{},\"cash\":{},\"inventory\":{},",
                "\"hero\":[{},{}],\"moving\":{},\"pending\":{},\"message\":{},\"queue\":{},",
                "\"verb\":\"{}\",\"input\":{},\"muted\":{},\"ticks\":{},\"dead\":{},\"flags\":{}}}"
            ),
            self.mode,
            st.scene,
            st.score,
            st.cash,
            json_str(&inv),
            self.hero.x,
            self.hero.y,
            self.hero.is_moving(),
            self.pending.is_some(),
            msg,
            self.queue.len(),
            VERBS[self.verb].0,
            json_str(&self.input),
            self.speaker.muted,
            self.ticks,
            st.dead,
            st.flags
        )
    }

    // ---------------------------------------------------------------- draw

    fn box_rect(lines: &[String], area: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
        let cols = lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(10) as i32;
        let w = cols * CW + 18;
        let h = lines.len() as i32 * LH + 14;
        let (ax0, ay0, ax1, ay1) = area;
        let x0 = ax0 + ((ax1 - ax0) - w) / 2;
        let y0 = (ay0 + ((ay1 - ay0) - h) / 2).max(ay0);
        (x0, y0, x0 + w, y0 + h)
    }

    /// A bordered box with text; `centered` lines, or left-aligned.
    fn text_box(
        f: &mut Frame,
        lines: &[String],
        area: (i32, i32, i32, i32),
        border: u8,
        title: Option<u8>,
        centered: bool,
    ) {
        let (x0, y0, x1, y1) = Self::box_rect(lines, area);
        f.message_box(x0, y0, x1, y1, WHITE, border);
        for (i, l) in lines.iter().enumerate() {
            let col = match title {
                Some(c) if i == 0 => c,
                _ => BLACK,
            };
            let y = y0 + 7 + i as i32 * LH;
            if centered {
                font::draw_centered(f, (x0 + x1) / 2, y, l, col, 1);
            } else {
                font::draw(f, x0 + 9, y, l, col);
            }
        }
    }

    fn buttons(f: &mut Frame, buttons: &Buttons, ys: (i32, i32)) {
        for (label, bx0, bx1) in buttons {
            f.message_box(*bx0, ys.0, *bx1, ys.1, WHITE, BLUE);
            font::draw_centered(f, (bx0 + bx1) / 2, ys.0 + 5, label, BLUE, 1);
        }
    }

    fn status_bar(&mut self, right: &str) {
        let f = &mut self.frame;
        f.fill_rect(0, 0, W, STATUS_H, WHITE);
        font::draw(f, 2, 1, &self.game.score_line(), BLACK);
        font::draw(f, W - 2 - font::text_width(right, 1), 1, right, BLACK);
        if self.speaker.muted {
            font::draw_centered(f, W / 2 + 8, 1, "(muted)", DARK_GRAY, 1);
        }
    }

    fn compose(&mut self) {
        let pic = match self.mode {
            Mode::Title | Mode::Quit => &self.title_pic,
            Mode::Dead => &self.tomb_pic,
            Mode::Playing => &self.pic,
            Mode::Ending => &self.ending_pic,
        };
        self.frame.copy_from(pic);
        let blink = (self.ticks / 8).is_multiple_of(2);
        match self.mode {
            Mode::Title | Mode::Quit => self.compose_title(blink),
            Mode::Playing => self.compose_playing(blink),
            Mode::Dead => self.compose_dead(),
            Mode::Ending => self.compose_ending(),
        }
        if let Some(m) = self.queue.front() {
            let lines = wrap(m, WRAP);
            Self::text_box(
                &mut self.frame,
                &lines,
                (0, STATUS_H, W, PIC_BOTTOM),
                RED,
                None,
                false,
            );
        }
    }

    fn compose_title(&mut self, blink: bool) {
        let f = &mut self.frame;
        let title = TITLE.to_uppercase();
        // A cheap neon glow: dark outline, then the bright tube.
        let x = W / 2 - font::text_width(&title, 4) / 2;
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (2, 2)] {
            font::draw_scaled(f, x + dx, 22 + dy, &title, MAGENTA, 4);
        }
        font::draw_scaled(f, x, 22, &title, LIGHT_MAGENTA, 4);
        let sx = W / 2 - font::text_width(SUBTITLE, 2) / 2;
        font::draw_shadowed(f, sx, 66, SUBTITLE, LIGHT_CYAN, BLUE, 2);
        let tag = "An original parody adventure";
        font::draw_shadowed(
            f,
            W / 2 - font::text_width(tag, 1) / 2,
            92,
            tag,
            WHITE,
            BLACK,
            1,
        );
        if self.mode == Mode::Title {
            if blink {
                let s = "Press ENTER or click to begin";
                font::draw_shadowed(
                    f,
                    W / 2 - font::text_width(s, 1) / 2,
                    176,
                    s,
                    YELLOW,
                    BLACK,
                    1,
                );
            }
            let s = if self.speaker.muted {
                "M: sound (off)"
            } else {
                "M: sound (on)"
            };
            font::draw_shadowed(
                f,
                W - 4 - font::text_width(s, 1),
                190,
                s,
                LIGHT_GRAY,
                BLACK,
                1,
            );
        } else {
            let lines: Vec<String> = [
                format!("Thanks for playing {TITLE}!"),
                String::new(),
                "Chip will be at the Rusty Anchor.".into(),
                "He's always there.".into(),
                String::new(),
                "Press ENTER for the title screen.".into(),
            ]
            .into();
            Self::text_box(f, &lines, (0, 104, W, 196), MAGENTA, Some(MAGENTA), true);
        }
    }

    fn compose_playing(&mut self, blink: bool) {
        let f = &mut self.frame;
        let st = &self.game.state;
        let mut placed = world::actors(st, self.ticks);
        let hf = if self.hero.is_moving() {
            self.hero.frame
        } else {
            0
        };
        placed.push(Placed::feet(
            hero_frame(self.hero.dir, hf),
            self.hero.x,
            self.hero.y,
            priority_for_y(self.hero.y),
        ));
        // Painter's order: farther (higher on screen) first.
        placed.sort_by_key(|p| (p.pri, p.y));
        for p in &placed {
            p.sprite.blit_priority(f, &self.pic, p.x, p.y, p.pri);
        }
        f.fill_rect(0, PIC_BOTTOM, W, H, BLACK);
        let (line, col) = if !self.input.is_empty() || self.last_cmd.is_none() {
            (
                format!(">{}{}", self.input, if blink { "_" } else { "" }),
                WHITE,
            )
        } else {
            (
                format!(">{}", self.last_cmd.as_deref().unwrap_or("")),
                LIGHT_GRAY,
            )
        };
        font::draw(f, 2, PIC_BOTTOM + 3, &line, col);
        let hint = format!(
            "Verb:[{}]  Right-click/Tab: change verb  F1: help",
            VERBS[self.verb].0
        );
        font::draw(f, 2, PIC_BOTTOM + 13, &hint, CYAN);
        let place = self.game.state.scene.name();
        self.status_bar(place);
    }

    fn compose_dead(&mut self) {
        self.status_bar("Deceased");
        let f = &mut self.frame;
        f.fill_rect(0, PIC_BOTTOM, W, H, BLACK);
        if let Some(d) = &self.death {
            let lines: Vec<String> = std::iter::once(d.cause.to_uppercase())
                .chain(std::iter::once(String::new()))
                .chain(d.epitaph.iter().map(|s| s.to_string()))
                .collect();
            Self::text_box(f, &lines, (120, 14, 316, 136), RED, Some(RED), true);
        }
        Self::buttons(f, &DEATH_BUTTONS, DEATH_BUTTON_Y);
        font::draw(
            f,
            2,
            PIC_BOTTOM + 3,
            "Keys:  R = Restore   N = Restart   Q = Quit",
            LIGHT_GRAY,
        );
        font::draw(
            f,
            2,
            PIC_BOTTOM + 13,
            "Choose wisely. It would be a first.",
            CYAN,
        );
    }

    fn compose_ending(&mut self) {
        self.status_bar("The End");
        let f = &mut self.frame;
        let bob = (self.ticks / 6 % 3) as u8;
        let hf = if (self.ticks / 40).is_multiple_of(2) {
            0
        } else {
            (self.ticks / 4 % 4) as u8
        };
        hero_frame(Dir::Down, hf).blit_priority(f, &self.ending_pic, 144, 168, 15);
        npcs::vonda(bob).blit_priority(f, &self.ending_pic, 178, 168, 15);
        let lines = self.game.ending_lines();
        Self::text_box(f, &lines, (0, 12, W, 134), MAGENTA, Some(MAGENTA), true);
        f.fill_rect(0, PIC_BOTTOM, W, H, BLACK);
        Self::buttons(f, &END_BUTTONS, END_BUTTON_Y);
    }

    fn render(&mut self) -> Result<(), JsValue> {
        self.compose();
        self.frame.to_rgba(&mut self.rgba);
        let img = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.rgba),
            WIDTH as u32,
            HEIGHT as u32,
        )?;
        self.ctx.put_image_data(&img, 0.0, 0.0)
    }
}

type FrameCallback = Closure<dyn FnMut(f64)>;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let doc = window().document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = doc
        .get_element_by_id("game")
        .ok_or("missing #game")?
        .dyn_into()?;
    canvas.set_width(WIDTH as u32);
    canvas.set_height(HEIGHT as u32);
    let ctx = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .dyn_into::<CanvasRenderingContext2d>()?;
    let search = window().location().search().unwrap_or_default();
    let test = search
        .trim_start_matches('?')
        .split('&')
        .any(|p| p == "test" || p.starts_with("test="));
    TEST_MODE.with(|t| t.set(test));
    let app = Rc::new(RefCell::new(App::new(ctx, canvas.clone())));
    APP.with(|a| *a.borrow_mut() = Some(app.clone()));

    {
        let app = app.clone();
        let cb = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            app.borrow_mut().on_key(&e)
        });
        window().add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        let app = app.clone();
        let cb = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            e.prevent_default();
            app.borrow_mut().on_mouse(&e)
        });
        canvas.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        let cb = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| e.prevent_default());
        canvas.add_event_listener_with_callback("contextmenu", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }

    // requestAnimationFrame loop.
    let f: Rc<RefCell<Option<FrameCallback>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move |t: f64| {
        app.borrow_mut().animate(t);
        if let Some(cb) = f.borrow().as_ref() {
            let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    window().request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    Ok(())
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// -------------------------------------------------------------------- test hooks
//
// Only usable when the page was opened with `?test`. The UI test suite
// (tests/ui) drives the game with real keyboard and mouse events and uses
// these to advance time deterministically and to inspect state and pixels.

fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Result<R, JsValue> {
    if !test_mode() {
        return Err(JsValue::from_str(
            "test hooks need the ?test query parameter",
        ));
    }
    APP.with(|a| a.borrow().clone())
        .map(|app| f(&mut app.borrow_mut()))
        .ok_or_else(|| JsValue::from_str("game not started"))
}

/// Game/UI state as JSON: mode, scene, score, current message, hero, etc.
#[wasm_bindgen]
pub fn test_state() -> Result<String, JsValue> {
    with_app(|a| a.state_json())
}

/// Advance the (frozen) game clock by `n` ticks of 50 ms.
#[wasm_bindgen]
pub fn test_step(n: u32) -> Result<(), JsValue> {
    with_app(|a| (0..n).for_each(|_| a.tick()))
}

/// Tick until the hero stops and any click action has run (max `max` ticks).
#[wasm_bindgen]
pub fn test_until_idle(max: u32) -> Result<u32, JsValue> {
    with_app(|a| a.until_idle(max))
}

/// The composed 320x200 frame as EGA palette indices (row-major).
#[wasm_bindgen]
pub fn test_framebuffer() -> Result<Vec<u8>, JsValue> {
    with_app(|a| {
        a.compose();
        a.frame.pixels.clone()
    })
}

/// The 16-color palette as 48 bytes of RGB.
#[wasm_bindgen]
pub fn test_palette() -> Vec<u8> {
    EGA.iter().flatten().copied().collect()
}
