//! Text helpers.

/// Greedy word-wrap to `width` columns. Explicit `\n` forces a break and a
/// blank line (`\n\n`) is preserved. Words longer than `width` are split.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let mut word = word.to_string();
            while word.chars().count() > width {
                if !line.is_empty() {
                    out.push(std::mem::take(&mut line));
                }
                let head: String = word.chars().take(width).collect();
                word = word.chars().skip(width).collect();
                out.push(head);
            }
            let need = if line.is_empty() {
                word.len()
            } else {
                line.len() + 1 + word.len()
            };
            if need > width && !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(&word);
        }
        out.push(line);
    }
    out
}

/// Wrap, then split into pages of at most `max_lines` lines (each page
/// re-joined with `\n`, so wrapping it again is a no-op).
pub fn paginate(text: &str, width: usize, max_lines: usize) -> Vec<String> {
    let mut lines = wrap(text, width);
    while lines.len() > 1 && lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
        .chunks(max_lines.max(1))
        .map(|c| {
            let mut c = c.to_vec();
            while c.len() > 1 && c[0].is_empty() {
                c.remove(0);
            }
            c.join("\n")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginates_long_messages() {
        let long = "word ".repeat(200);
        let pages = paginate(&long, 38, 15);
        assert!(pages.len() > 1);
        for p in &pages {
            let w = wrap(p, 38);
            assert!(w.len() <= 15);
            assert_eq!(&w.join("\n"), p, "re-wrapping a page changes nothing");
        }
        assert_eq!(paginate("short", 38, 15), vec!["short".to_string()]);
        let help = paginate(crate::game::logic::HELP, 38, 15);
        assert!(help.len() >= 2);
    }

    #[test]
    fn wraps_on_word_boundaries() {
        let w = wrap("the quick brown fox jumps over the lazy dog", 10);
        assert!(w.iter().all(|l| l.len() <= 10), "{w:?}");
        assert_eq!(w.join(" "), "the quick brown fox jumps over the lazy dog");
    }

    #[test]
    fn keeps_explicit_breaks_and_splits_long_words() {
        let w = wrap("a\n\nb", 10);
        assert_eq!(w, vec!["a", "", "b"]);
        let w = wrap("abcdefghijkl", 5);
        assert_eq!(w, vec!["abcde", "fghij", "kl"]);
    }
}
