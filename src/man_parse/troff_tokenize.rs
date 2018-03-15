// implementing tokenization logic for a very small subset of Troff

use simple_parser::token::{Token, TokenClass, TokenGenerator};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TroffToken {
    Macro,
    TextWord,
    DoubleQuote,
    Backslash,

    // signifies that the previous Token
    // was a backslash, so this token
    // is being modified by the slash.
    // i.e. if this character is a '-',
    // then the /- signifies 'escaped -'
    // if this character is the letter 'f',
    // then /f means "set the font to..."
    EscapeCommand,
}

// TODO: implement #[derive(TokenClass)] macro...
impl TokenClass for TroffToken {}

pub struct TroffClassifier;

impl TokenGenerator<TroffToken> for TroffClassifier {
    fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<TroffToken>> {
        let mut tokens = Vec::new(); // TODO: preallocate a smart amount

        if starts_line && word.starts_with('.') {
            let tok = Token::new(TroffToken::Macro, word.to_owned(), true);
            tokens.push(tok);
            return tokens;
        }

        // we're going to redefine this
        // so we can say starts_line is false
        // after we do a split
        let mut starts_line = starts_line;

        let mut peek_iter = word.chars().enumerate().peekable();

        let mut base_index: usize = 0;
        while let Some((walker, c)) = peek_iter.next() {
            if let Some(special_class) = try_match_special(&c) {
                // found a special char in the middle of the word
                // so split here
                if walker > base_index {
                    let prev_word = word[base_index..walker].to_owned();
                    let prev_tok = Token::new(TroffToken::TextWord, prev_word, starts_line);
                    tokens.push(prev_tok);
                    starts_line = false;
                }

                let special_tok = Token::new(special_class, c.to_string(), starts_line);

                tokens.push(special_tok);
                starts_line = false;
                base_index = walker + 1;

                // backslash has special behavior
                // \- prints an em-dash
                // \fBHello prints 'Hello' in bold
                if special_class == TroffToken::Backslash {
                    if let Some(next_char) = peek_iter.peek() {
                        // how long is the escape?
                        // 1 char: \-
                        // 2 char: \fB[text]
                        // 3+ char: \f(fn... ??
                        let char_index = next_char.0;
                        let escaped = get_escaped(&word[char_index..word.len()]);

                        if let Some(escape_class) = try_match_escaped(&next_char.1) {
                            let escaped_token =
                                Token::new(escape_class, next_char.1.to_string(), starts_line);
                            tokens.push(escaped_token);
                            starts_line = false;
                        }
                    }
                }
            }
        }

        if base_index < word.len() {
            let word_tok = Token::new(
                TroffToken::TextWord,
                word[base_index..word.len()].to_owned(),
                starts_line,
            );
            tokens.push(word_tok);
        }

        tokens
    }

    fn is_comment(&self, word: &str) -> bool {
        word.starts_with("\\\"") || word.starts_with(".\\\"") || word.starts_with("\\#")
            || word == "."
    }
}

/// Given a str slice that follows an escape,
/// return the substring of the slice that is
/// affected by the escape.
fn get_escaped(s: &str) -> &str {
    if s.starts_with("B") {
        return &s[0..1];
    } else {
        return &s[0..1];
    }
}

fn try_match_escaped(c: &char) -> Option<TroffToken> {
    match *c {
        'f' => Some(TroffToken::EscapeCommand),
        _ => None,
    }
}

fn try_match_special(c: &char) -> Option<TroffToken> {
    let special_tok = match *c {
        '\\' => Some(TroffToken::Backslash),
        '"' => Some(TroffToken::DoubleQuote),
        _ => None,
    };

    special_tok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_word() {
        let word = "\"MyBigWord\""; // "MyBigWord"
        let generator = TroffClassifier {};

        let actual = generator.generate(word, false);

        let expected = vec![
            Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false),
            Token::new(TroffToken::TextWord, "MyBigWord".to_owned(), false),
            Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false),
        ];

        assert!(
            actual == expected,
            "expected: {:?}\nactual: {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn test_short_word() {
        let word = "\"I\""; // "I"
        let generator = TroffClassifier {};

        let actual = generator.generate(word, false);

        let expected = vec![
            Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false),
            Token::new(TroffToken::TextWord, "I".to_owned(), false),
            Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false),
        ];

        assert!(
            actual == expected,
            "expected: {:?}\nactual: {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn test_quote_only() {
        let word = "\"";
        let generator = TroffClassifier {};

        let actual = generator.generate(word, false);

        let expected = vec![Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false)];

        assert!(
            actual == expected,
            "expected: {:?}\nactual: {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn test_double_quote() {
        let word = "\"\"";
        let generator = TroffClassifier {};

        let actual = generator.generate(word, false);

        let expected = vec![
            Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false),
            Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false),
        ];

        assert!(
            actual == expected,
            "expected: {:?}\nactual: {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn test_multiple_splits() {
        let word = "\\lefthalf\\righthalf\\";
        let generator = TroffClassifier {};

        let actual = generator.generate(word, true);

        let expected = vec![
            Token::new(TroffToken::Backslash, "\\".to_owned(), true),
            Token::new(TroffToken::TextWord, "lefthalf".to_owned(), false),
            Token::new(TroffToken::Backslash, "\\".to_owned(), false),
            Token::new(TroffToken::TextWord, "righthalf".to_owned(), false),
            Token::new(TroffToken::Backslash, "\\".to_owned(), false),
        ];

        assert!(
            actual == expected,
            "expected: {:?}\nactual: {:?}",
            expected,
            actual
        );
    }
}
