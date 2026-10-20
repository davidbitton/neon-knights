//! Browser front-end: canvas output, keyboard/mouse input, localStorage.
//! All game rules live in `crate::game`; this file only wires them to the DOM.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, KeyboardEvent, MouseEvent, Storage, Window};

use crate::engine::actor::Actor;
use crate::engine::gfx::{priority_for_y, Frame, Picture, HEIGHT, WIDTH};
use crate::engine::palette::{self, *};
use crate::engine::sprite::{gus_frame, hero_frame, Dir, Sprite};
use crate::game::logic::{Death, Effect, HELP, INTRO};
use crate::game::scene::{self, PIC_BOTTOM, STATUS_H};
use crate::game::state::{GameState, START_POS};
use crate::game::text::wrap;
use crate::game::Game;

const SCALE: f64 = 3.0;
const SAVE_KEY: &str = "neon-nights.save.v1";
const TICK_MS: f64 = 50.0;
/// Text cell size in framebuffer pixels.
const CW: i32 = 6;
const LH: i32 = 9;
const WRAP: usize = 38;
const MAX_INPUT: usize = 44;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Title,
    Playing,
    Dead,
    Quit,
}

/// Click verbs (right-click / Tab cycles them).
const VERBS: [(&str, &str); 5] = [("Walk", ""), ("Look", "look"), ("Take", "take"), ("Talk", "talk to"), ("Use", "use")];

const BUTTONS: [(&str, i32, i32); 3] = [("Restore [R]", 122, 186), ("Restart [N]", 189, 253), ("Quit [Q]", 256, 314)];
const BUTTON_Y: (i32, i32) = (140, 158);

struct App {
    mode: Mode,
    game: Game,
    hero: Actor,
    bar_pic: Picture,
    bar_flags: u32,
    title_pic: Picture,
    tomb_pic: Picture,
    frame: Frame,
    rgba: Vec<u8>,
    input: String,
    last_cmd: Option<String>,
    queue: VecDeque<String>,
    death: Option<Death>,
    death_pending: bool,
    verb: usize,
    pending: Option<String>,
    ticks: u64,
    acc: f64,
    last_t: Option<f64>,
    lo: CanvasRenderingContext2d,
    hi: CanvasRenderingContext2d,
    hi_canvas: HtmlCanvasElement,
    font: String,
}

fn window() -> Window {
    web_sys::window().expect("no window")
}

fn storage() -> Option<Storage> {
    window().local_storage().ok().flatten()
}

fn ctx2d(c: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    c.get_context("2d")?
        .ok_or_else(|| JsValue::from_str("no 2d context"))?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| JsValue::from_str("not a 2d context"))
}

fn note_sprite() -> Sprite {
    Sprite::from_rows(&["..YY", "..Y.", "..Y.", "YYY.", "YYY."])
}

fn z_sprite() -> Sprite {
    Sprite::from_rows(&["WWW", "..W", ".W.", "WWW"])
}

impl App {
    fn new(lo: CanvasRenderingContext2d, hi: CanvasRenderingContext2d, hi_canvas: HtmlCanvasElement) -> App {
        let game = Game::new();
        let bar_pic = scene::draw_bar(&game.state);
        let bar_flags = game.state.flags;
        // Pick the largest monospace size whose glyphs fit our text cell.
        let mut font = String::new();
        for px in (12..=28).rev() {
            font = format!("bold {px}px 'DejaVu Sans Mono', 'Courier New', monospace");
            hi.set_font(&font);
            let w = hi.measure_text("M").map(|m| m.width()).unwrap_or(99.0);
            if w <= CW as f64 * SCALE {
                break;
            }
        }
        App {
            mode: Mode::Title,
            hero: Actor::new(START_POS.0, START_POS.1),
            game,
            bar_pic,
            bar_flags,
            title_pic: scene::draw_title(),
            tomb_pic: scene::draw_tomb(),
            frame: Frame::new(),
            rgba: Vec::with_capacity(WIDTH * HEIGHT * 4),
            input: String::new(),
            last_cmd: None,
            queue: VecDeque::new(),
            death: None,
            death_pending: false,
            verb: 1,
            pending: None,
            ticks: 0,
            acc: 0.0,
            last_t: None,
            lo,
            hi,
            hi_canvas,
            font,
        }
    }

    // ---------------------------------------------------------------- flow

    fn restart(&mut self) {
        self.game = Game::new();
        self.hero = Actor::new(START_POS.0, START_POS.1);
        self.mode = Mode::Playing;
        self.queue.clear();
        self.queue.push_back(INTRO.to_string());
        self.death = None;
        self.death_pending = false;
        self.pending = None;
        self.input.clear();
        self.last_cmd = None;
    }

    fn save(&mut self) {
        self.game.state.hero = (self.hero.x, self.hero.y);
        let data = self.game.state.serialize();
        let ok = storage().map(|s| s.set_item(SAVE_KEY, &data).is_ok()).unwrap_or(false);
        self.queue.push_back(if ok {
            "Game saved. Your progress is safe, which is more than can be said for your liver.".into()
        } else {
            "Couldn't save: this browser won't let us use localStorage. Chip will have to rely on his memory. Uh-oh.".into()
        });
    }

    fn restore(&mut self) {
        let data = storage().and_then(|s| s.get_item(SAVE_KEY).ok().flatten());
        match data.map(|d| GameState::deserialize(&d)) {
            Some(Ok(st)) => {
                self.hero = Actor::new(st.hero.0, st.hero.1);
                self.game.state = st;
                self.mode = Mode::Playing;
                self.death = None;
                self.death_pending = false;
                self.pending = None;
                self.queue.clear();
                self.queue.push_back("Game restored. Previously, on Neon Nights...".into());
            }
            Some(Err(_)) => self.queue.push_back("That saved game is corrupted. Much like this town.".into()),
            None => self
                .queue
                .push_back("There's no saved game. Chip lives in the moment, mostly because he forgets things.".into()),
        }
    }

    fn submit(&mut self, line: &str) {
        let line = line.trim().to_string();
        self.game.state.hero = (self.hero.x, self.hero.y);
        let reply = self.game.input(&line);
        self.last_cmd = Some(line);
        self.queue.extend(reply.messages);
        match reply.effect {
            Effect::None | Effect::Scored(_) => {}
            Effect::Died(d) => {
                self.hero.stop();
                self.pending = None;
                self.death = Some(d);
                self.death_pending = true;
            }
            Effect::Save => self.save(),
            Effect::Restore => self.restore(),
            Effect::Restart => self.restart(),
            Effect::Quit => {
                self.queue.clear();
                self.mode = Mode::Quit;
            }
        }
    }

    fn dismiss(&mut self) {
        self.queue.pop_front();
        if self.queue.is_empty() && self.death_pending {
            self.death_pending = false;
            self.mode = Mode::Dead;
        }
    }

    // ---------------------------------------------------------------- input

    fn on_key(&mut self, e: &KeyboardEvent) {
        if e.ctrl_key() || e.meta_key() || e.alt_key() {
            return;
        }
        let key = e.key();
        let mut handled = true;
        match self.mode {
            Mode::Title => {
                if key == "Enter" || key == " " {
                    self.restart();
                } else {
                    handled = false;
                }
            }
            Mode::Quit => {
                if key == "Enter" || key == " " {
                    self.mode = Mode::Title;
                } else {
                    handled = false;
                }
            }
            Mode::Dead => {
                if !self.queue.is_empty() {
                    if matches!(key.as_str(), "Enter" | "Escape" | " ") {
                        self.dismiss();
                    }
                } else {
                    match key.to_ascii_lowercase().as_str() {
                        "r" => self.restore(),
                        "n" | "enter" => self.restart(),
                        "q" => self.mode = Mode::Quit,
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
                        "F1" => self.queue.push_back(HELP.to_string()),
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

    fn arrow(&mut self, d: Dir) {
        self.pending = None;
        self.hero.press_dir(d);
    }

    fn to_fb(&self, e: &MouseEvent) -> (i32, i32) {
        let r = self.hi_canvas.get_bounding_client_rect();
        let x = ((e.client_x() as f64 - r.left()) / r.width() * WIDTH as f64).floor() as i32;
        let y = ((e.client_y() as f64 - r.top()) / r.height() * HEIGHT as f64).floor() as i32;
        (x, y)
    }

    fn on_mouse(&mut self, e: &MouseEvent) {
        let (x, y) = self.to_fb(e);
        let right = e.button() == 2;
        match self.mode {
            Mode::Title => self.restart(),
            Mode::Quit => self.mode = Mode::Title,
            Mode::Dead => {
                if !self.queue.is_empty() {
                    self.dismiss();
                } else if y >= BUTTON_Y.0 && y < BUTTON_Y.1 {
                    for (i, (_, x0, x1)) in BUTTONS.iter().enumerate() {
                        if x >= *x0 && x < *x1 {
                            match i {
                                0 => self.restore(),
                                1 => self.restart(),
                                _ => self.mode = Mode::Quit,
                            }
                        }
                    }
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
                let hot = if self.verb == 0 { None } else { scene::hotspot_at(&self.game.state, x, y) };
                match hot {
                    Some(h) => {
                        let (_, word) = VERBS[self.verb];
                        let cmd = format!("{word} {}", scene::noun_word(h.noun));
                        if self.verb == 1 || scene::is_near(&self.game.state, h.noun) {
                            self.submit(&cmd);
                        } else {
                            self.hero.walk_to(h.approach.0, h.approach.1);
                            self.pending = Some(cmd);
                        }
                    }
                    None => {
                        self.pending = None;
                        self.hero.walk_to(x, y);
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
        if self.game.state.flags != self.bar_flags {
            self.bar_pic = scene::draw_bar(&self.game.state);
            self.bar_flags = self.game.state.flags;
        }
        self.hero.tick(&self.bar_pic);
        self.game.state.hero = (self.hero.x, self.hero.y);
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
        while self.acc >= TICK_MS {
            self.acc -= TICK_MS;
            self.tick();
        }
        if let Err(e) = self.render() {
            web_sys::console::error_1(&e);
        }
    }

    // ---------------------------------------------------------------- draw

    fn message_lines(&self) -> Option<Vec<String>> {
        self.queue.front().map(|m| wrap(m, WRAP))
    }

    fn box_rect(lines: &[String], area: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
        let cols = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(10) as i32;
        let w = cols * CW + 18;
        let h = lines.len() as i32 * LH + 14;
        let (ax0, ay0, ax1, ay1) = area;
        let x0 = ax0 + ((ax1 - ax0) - w) / 2;
        let y0 = (ay0 + ((ay1 - ay0) - h) / 2).max(ay0);
        (x0, y0, x0 + w, y0 + h)
    }

    fn compose(&mut self) {
        let pic = match self.mode {
            Mode::Title | Mode::Quit => &self.title_pic,
            Mode::Dead => &self.tomb_pic,
            Mode::Playing => &self.bar_pic,
        };
        self.frame.copy_from(pic);
        let f = &mut self.frame;
        if self.mode == Mode::Playing {
            let st = &self.game.state;
            if st.flag(crate::game::Flag::PlayedJukebox) {
                let cols = [LIGHT_CYAN, LIGHT_MAGENTA, YELLOW, LIGHT_GREEN];
                let c = cols[(self.ticks / 4 % 4) as usize];
                f.fill_rect(226, 70, 247, 78, c);
                let rise = (self.ticks % 30) as i32;
                note_sprite().blit(f, 240 + rise / 6, 54 - rise);
            }
            let zr = (self.ticks % 40) as i32;
            if zr < 30 {
                z_sprite().blit(f, 292 + zr / 6, 104 - zr / 2);
            }
            gus_frame((self.ticks / 12 % 2) as u8).blit_priority(f, &self.bar_pic, scene::GUS_POS.0, scene::GUS_POS.1, priority_for_y(scene::GUS_POS.1));
            let hf = if self.hero.is_moving() { self.hero.frame } else { 0 };
            hero_frame(self.hero.dir, hf).blit_priority(f, &self.bar_pic, self.hero.x, self.hero.y, priority_for_y(self.hero.y));
        }
        if matches!(self.mode, Mode::Playing | Mode::Dead) {
            f.fill_rect(0, 0, WIDTH as i32, STATUS_H, WHITE);
            f.fill_rect(0, PIC_BOTTOM, WIDTH as i32, HEIGHT as i32, BLACK);
        }
        if self.mode == Mode::Dead {
            if let Some(d) = &self.death {
                let lines: Vec<String> = std::iter::once(d.cause.to_uppercase()).chain(std::iter::once(String::new())).chain(d.epitaph.iter().map(|s| s.to_string())).collect();
                let (x0, y0, x1, y1) = Self::box_rect(&lines, (120, 14, 316, 136));
                f.message_box(x0, y0, x1, y1, WHITE, RED);
            }
            for (_, bx0, bx1) in BUTTONS {
                f.message_box(bx0, BUTTON_Y.0, bx1, BUTTON_Y.1, WHITE, BLUE);
            }
        }
        if let Some(lines) = self.message_lines() {
            let (x0, y0, x1, y1) = Self::box_rect(&lines, (0, STATUS_H, WIDTH as i32, PIC_BOTTOM));
            self.frame.message_box(x0, y0, x1, y1, WHITE, RED);
        }
    }

    fn text(&self, s: &str, x: i32, y: i32, color: u8) {
        self.hi.set_fill_style_str(&palette::css(color));
        let _ = self.hi.fill_text(s, x as f64 * SCALE, y as f64 * SCALE + 2.0);
    }

    fn render(&mut self) -> Result<(), JsValue> {
        self.compose();
        self.frame.to_rgba(&mut self.rgba);
        let img = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&self.rgba), WIDTH as u32, HEIGHT as u32)?;
        self.lo.put_image_data(&img, 0.0, 0.0)?;

        let hi = &self.hi;
        hi.clear_rect(0.0, 0.0, WIDTH as f64 * SCALE, HEIGHT as f64 * SCALE);
        hi.set_text_baseline("top");
        hi.set_font(&self.font);
        hi.set_shadow_blur(0.0);
        let blink = (self.ticks / 8) % 2 == 0;

        match self.mode {
            Mode::Title => {
                hi.set_text_align("center");
                hi.set_shadow_color("#ff55ff");
                hi.set_shadow_blur(24.0);
                hi.set_font("bold 96px 'DejaVu Sans Mono', monospace");
                hi.set_fill_style_str(&palette::css(LIGHT_MAGENTA));
                hi.fill_text("NEON NIGHTS", 480.0, 120.0)?;
                hi.set_shadow_color("#55ffff");
                hi.set_font("bold 34px 'DejaVu Sans Mono', monospace");
                hi.set_fill_style_str(&palette::css(LIGHT_CYAN));
                hi.fill_text("A Gentleman's Evening Out", 480.0, 230.0)?;
                hi.set_shadow_blur(0.0);
                hi.set_font(&self.font);
                hi.set_fill_style_str(&palette::css(WHITE));
                hi.fill_text("An original parody adventure  -  Milestone 1", 480.0, 290.0)?;
                if blink {
                    hi.set_fill_style_str(&palette::css(YELLOW));
                    hi.fill_text("Press ENTER or click to begin", 480.0, 530.0)?;
                }
                hi.set_text_align("left");
            }
            Mode::Quit => {
                hi.set_fill_style_str("rgba(0,0,0,0.7)");
                hi.fill_rect(0.0, 0.0, 960.0, 600.0);
                hi.set_text_align("center");
                hi.set_fill_style_str(&palette::css(LIGHT_MAGENTA));
                hi.fill_text("Thanks for playing Neon Nights!", 480.0, 220.0)?;
                hi.set_fill_style_str(&palette::css(WHITE));
                hi.fill_text("Chip will be at the Rusty Anchor. He's always at the Rusty Anchor.", 480.0, 270.0)?;
                hi.set_fill_style_str(&palette::css(YELLOW));
                hi.fill_text("Press ENTER to return to the title screen.", 480.0, 340.0)?;
                hi.set_text_align("left");
            }
            Mode::Playing | Mode::Dead => {
                let score = self.game.score_line();
                self.text(&score, 2, 0, BLACK);
                let place = if self.mode == Mode::Dead { "Deceased" } else { "The Rusty Anchor" };
                self.text(place, WIDTH as i32 - 2 - place.len() as i32 * CW, 0, BLACK);
                if self.mode == Mode::Playing {
                    let (line, col) = if !self.input.is_empty() || self.last_cmd.is_none() {
                        (format!(">{}{}", self.input, if blink { "_" } else { "" }), WHITE)
                    } else {
                        (format!(">{}", self.last_cmd.as_deref().unwrap_or("")), LIGHT_GRAY)
                    };
                    self.text(&line, 2, PIC_BOTTOM + 3, col);
                    let hint = format!("Click verb: {}   (right-click/Tab to change, F1 help)", VERBS[self.verb].0);
                    self.text(&hint, 2, PIC_BOTTOM + 13, CYAN);
                } else {
                    self.text("You have died. Choose wisely; it's a first for you.", 2, PIC_BOTTOM + 7, LIGHT_GRAY);
                }
                if self.mode == Mode::Dead {
                    if let Some(d) = &self.death {
                        let lines: Vec<String> = std::iter::once(d.cause.to_uppercase()).chain(std::iter::once(String::new())).chain(d.epitaph.iter().map(|s| s.to_string())).collect();
                        let (x0, y0, x1, _) = Self::box_rect(&lines, (120, 14, 316, 136));
                        for (i, l) in lines.iter().enumerate() {
                            let w = l.chars().count() as i32 * CW;
                            let col = if i == 0 { RED } else { BLACK };
                            self.text(l, x0 + (x1 - x0 - w) / 2, y0 + 7 + i as i32 * LH, col);
                        }
                    }
                    for (label, bx0, bx1) in BUTTONS {
                        let w = label.len() as i32 * CW;
                        self.text(label, bx0 + (bx1 - bx0 - w) / 2, BUTTON_Y.0 + 5, BLUE);
                    }
                }
                if let Some(lines) = self.message_lines() {
                    let (x0, y0, _, _) = Self::box_rect(&lines, (0, STATUS_H, WIDTH as i32, PIC_BOTTOM));
                    for (i, l) in lines.iter().enumerate() {
                        self.text(l, x0 + 9, y0 + 7 + i as i32 * LH, BLACK);
                    }
                }
            }
        }
        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let doc = window().document().ok_or("no document")?;
    let lo_canvas: HtmlCanvasElement = doc.get_element_by_id("lo").ok_or("missing #lo")?.dyn_into()?;
    let hi_canvas: HtmlCanvasElement = doc.get_element_by_id("hi").ok_or("missing #hi")?.dyn_into()?;
    lo_canvas.set_width(WIDTH as u32);
    lo_canvas.set_height(HEIGHT as u32);
    hi_canvas.set_width((WIDTH as f64 * SCALE) as u32);
    hi_canvas.set_height((HEIGHT as f64 * SCALE) as u32);
    let lo = ctx2d(&lo_canvas)?;
    let hi = ctx2d(&hi_canvas)?;
    let app = Rc::new(RefCell::new(App::new(lo, hi, hi_canvas.clone())));

    {
        let app = app.clone();
        let cb = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| app.borrow_mut().on_key(&e));
        window().add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        let app = app.clone();
        let cb = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            e.prevent_default();
            app.borrow_mut().on_mouse(&e)
        });
        hi_canvas.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        let cb = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| e.prevent_default());
        hi_canvas.add_event_listener_with_callback("contextmenu", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }

    // requestAnimationFrame loop.
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
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
