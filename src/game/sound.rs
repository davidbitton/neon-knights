//! PC-speaker-style sound effects: one square-wave voice, tunes as
//! (frequency in Hz, duration in ms) pairs. A frequency of 0 is a rest.
//! All tunes are original. The web layer plays them via Web Audio.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    /// Title jingle.
    Title,
    /// A point (or several) scored.
    Score,
    /// The tombstone appears.
    Death,
    /// The ending screen.
    Ending,
    /// The parser didn't understand.
    Buzz,
}

impl Sfx {
    pub const ALL: [Sfx; 5] = [Sfx::Title, Sfx::Score, Sfx::Death, Sfx::Ending, Sfx::Buzz];
}

pub type Note = (f32, u16);

// Equal-tempered pitches we use (A4 = 440 Hz).
const B3: f32 = 246.94;
const C4: f32 = 261.63;
const CS4: f32 = 277.18;
const D4: f32 = 293.66;
const F4: f32 = 349.23;
const A4: f32 = 440.00;
const C5: f32 = 523.25;
const E5: f32 = 659.25;
const G5: f32 = 783.99;
const A5: f32 = 880.00;
const B5: f32 = 987.77;
const C6: f32 = 1046.50;
const D6: f32 = 1174.66;
const E6: f32 = 1318.51;
const G6: f32 = 1567.98;
const R: f32 = 0.0;

/// A swaggering little lounge lick.
const TITLE: [Note; 10] = [
    (E5, 110),
    (G5, 110),
    (B5, 110),
    (E6, 220),
    (R, 60),
    (D6, 110),
    (B5, 110),
    (G5, 110),
    (A5, 330),
    (E5, 220),
];

/// Two quick rising chirps.
const SCORE: [Note; 2] = [(C6, 50), (G6, 90)];

/// A slow, sinking sigh.
const DEATH: [Note; 9] = [
    (A4, 260),
    (F4, 260),
    (D4, 260),
    (CS4, 520),
    (R, 90),
    (D4, 180),
    (CS4, 180),
    (C4, 180),
    (B3, 720),
];

/// Ta-ta-ta-TAAA, with a cheesy flourish.
const ENDING: [Note; 11] = [
    (C5, 100),
    (C5, 100),
    (C5, 100),
    (G5, 320),
    (R, 40),
    (E5, 100),
    (G5, 100),
    (C6, 420),
    (R, 60),
    (B5, 110),
    (C6, 640),
];

/// A low, grumpy buzz.
const BUZZ: [Note; 2] = [(98.0, 90), (87.31, 140)];

pub fn notes(sfx: Sfx) -> &'static [Note] {
    match sfx {
        Sfx::Title => &TITLE,
        Sfx::Score => &SCORE,
        Sfx::Death => &DEATH,
        Sfx::Ending => &ENDING,
        Sfx::Buzz => &BUZZ,
    }
}

/// Total length in milliseconds.
pub fn duration_ms(sfx: Sfx) -> u32 {
    notes(sfx).iter().map(|n| n.1 as u32).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunes_are_short_audible_and_distinct() {
        for s in Sfx::ALL {
            let n = notes(s);
            assert!(!n.is_empty());
            assert!(n.iter().any(|(f, _)| *f > 0.0));
            assert!(n
                .iter()
                .all(|(f, d)| (*f == 0.0 || (60.0..4000.0).contains(f)) && *d >= 30));
            assert!(duration_ms(s) <= 3500, "{s:?} is too long");
        }
        assert!(duration_ms(Sfx::Score) < 200, "the blip must not drag");
        for (i, a) in Sfx::ALL.iter().enumerate() {
            for b in &Sfx::ALL[i + 1..] {
                assert_ne!(notes(*a), notes(*b));
            }
        }
    }
}
