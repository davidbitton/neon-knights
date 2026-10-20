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

#[cfg(test)]
mod tests {
    use super::*;

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
