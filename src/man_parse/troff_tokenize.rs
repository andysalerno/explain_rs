// implementing tokenization logic for a very small subset of Troff

use simple_parser::token::{Token, TokenClass, TokenGenerator};

#[derive(Debug, PartialEq)]
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
    SlashEscaped,
}

// TODO: implement #[derive(TokenClass)] macro...
impl TokenClass for TroffToken {}

pub struct TroffClassifier;

impl TokenGenerator<TroffToken> for TroffClassifier {
    fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<TroffToken>> {
        let mut tokens = Vec::new();

        if starts_line && word.starts_with('.') {
            let tok = Token::new(TroffToken::Macro, word.to_owned(), true);
            tokens.push(tok);
            return tokens;
        }

        // we're going to redefine this
        // so we can say starts_line is false
        // after we do a split
        let mut starts_line = starts_line;

        let mut base_index: usize = 0;
        let chars = word.chars();
        for (walker, c) in chars.enumerate() {
            // if the last character was a backslash-escape,
            // the current character is the escaped token
            if tokens.last().is_some() && tokens.last().unwrap().class == TroffToken::Backslash {
                let escaped_tok = Token::new(TroffToken::SlashEscaped, c.to_string(), starts_line);
                tokens.push(escaped_tok);
                base_index = walker + 1;
                continue;
            }

            // first, see if this char is any special escape char
            let special_tok = match try_match_special(&c) {
                Some(t) => Some(Token::new(t, c.to_string(), starts_line)),
                None => None,
            };

            if special_tok.is_some() {
                if walker > base_index {
                    let prev_word = word[base_index..walker].to_owned();
                    let prev_tok = Token::new(TroffToken::TextWord, prev_word, starts_line);
                    tokens.push(prev_tok);
                }

                tokens.push(special_tok.unwrap());
                starts_line = false;
                base_index = walker + 1;
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
