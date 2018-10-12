#[derive(Default)]
pub struct LineInfo {
    whitespace_len: usize,
    nonwhitespace_len: usize,
}

pub enum LengthRule {
    Whitespace,
    NonWhitespace,
    Everything,
}

impl LineInfo {
    pub fn len(&self, rule: LengthRule) -> usize {
        match rule {
            LengthRule::Everything => self.whitespace_len + self.nonwhitespace_len,
            LengthRule::NonWhitespace => self.nonwhitespace_len,
            LengthRule::Whitespace => self.whitespace_len,
        }
    }

    /// Increase the line length information based on the content of the slice argument.
    /// Note: currently only the first char in the word is considered when judging between
    /// whitespace and non-whitespace, so the word is expected to contain exclusively one or the other,
    /// and not a mix.
    pub fn increase_len(&mut self, word: &str) {
        if word.len() == 0 {
            return;
        }

        let first_char = word.chars().next().unwrap();

        if first_char.is_whitespace() {
            self.whitespace_len += word.len();
        } else {
            self.nonwhitespace_len += word.len();
        }
    }

    /// True if our length consists of only whitespace (or nothing at all).
    pub fn is_whitespace_only(&self) -> bool {
        self.nonwhitespace_len == 0
    }

    /// Reset all data so the lengths are all zero.
    pub fn reset(&mut self) {
        self.whitespace_len = 0;
        self.nonwhitespace_len = 0;
    }
}
