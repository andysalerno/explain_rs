// implementing tokenization logic for a very small subset of Troff

use simple_parser::token::{Token, TokenClass};
use simple_parser::token_generator::TokenGenerator;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TroffToken {
    Space,
    Macro,
    TextWord,
    DoubleQuote,
    Backslash,

    // a command that immediately follows '\'
    // i.e., the 'f' in '\f'
    EscapeCommand,

    // an argument to a command
    // i.e., the 'B' in '\fB',
    // or the 'I' in '.ft I'
    CommandArg,
    ArgOpenBracket,
    ArgCloseBracket,
    ArgOpenParen,

    EmptyLine,
}

// Flag our TroffToken class with unit trait to prove it is a Token
impl TokenClass for TroffToken {}

const SPACE: &str = " ";

pub struct TroffTokenGenerator;

impl TokenGenerator<TroffToken> for TroffTokenGenerator {
    fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<TroffToken>> {
        let mut tokens = Vec::new(); // TODO: preallocate a smart amount

        if starts_line && word.starts_with('.') {
            let tok = Token::new(TroffToken::Macro, word.to_owned(), true);
            tokens.push(tok);
            return tokens;
        }

        if starts_line && word.len() == 0 {
            let empty_line = Token::new(TroffToken::EmptyLine, "".into(), true);
            tokens.push(empty_line);
            return tokens;
        }

        let mut starts_line = starts_line;
        let mut char_iter = word.chars().enumerate();
        let mut base_index: usize = 0;

        while let Some((c_index, c)) = char_iter.next() {
            if let Some(special_class) = try_match_special(&c) {
                // Some chars are special chars that are tokens on their own
                // and appear in the middle of words, so in this case we split the word into multiple tokens.
                if c_index > base_index {
                    // everything before the split is a token on its own
                    let prev_word = word[base_index..c_index].to_owned();
                    let prev_tok = Token::new(TroffToken::TextWord, prev_word, starts_line);
                    tokens.push(prev_tok);
                    starts_line = false;
                }

                // after the split, we have the special token itself
                let special_tok = Token::new(special_class, c.to_string(), starts_line);

                tokens.push(special_tok);
                starts_line = false;
                base_index = c_index + 1;

                // after a '\' is always an escaped character
                if special_class == TroffToken::Backslash {
                    let next = {
                        if let Some(n) = char_iter.next() {
                            n
                        } else {
                            continue;
                        }
                    };

                    let (next_index, command_char) = next;

                    let escaped =
                        Token::new(TroffToken::EscapeCommand, command_char.to_string(), false);

                    tokens.push(escaped);
                    base_index = next_index + 1;

                    // after the escaped token, we might now have an argument
                    // i.e., '\fB' takes arg 'B'
                    if !command_has_args(command_char) {
                        continue;
                    }

                    let mut advance_count = 0;
                    let args = get_escaped_args(&word[next_index + 1..]);

                    for arg in args {
                        advance_count += arg.value.len();
                        tokens.push(arg);
                    }

                    for _ in 0..advance_count {
                        char_iter.next();
                    }

                    base_index += advance_count;
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
        word.starts_with("\\\"")
            || word.starts_with(".\\\"")
            || word.starts_with("\\#")
            || word == "."
    }

    fn space_tok(&self) -> Option<Token<TroffToken>> {
        Some(Token::new(TroffToken::Space, SPACE.to_owned(), false))
    }
}

/// Given an escaped command char, does it have args?
/// I.e., \fBboldword is:
/// an escape char \,
/// a command char f,
/// an argument B,
/// and a word boldword.
/// So an escaped 'f' may have args.
/// However, \- is simply the escape for '-',
/// so it does not have args.
fn command_has_args(command: char) -> bool {
    match command {
        'f' | 'm' => true,
        _ => false,
    }
}

/// Given a word that appears after an escaped char,
/// split the word as necessary into the argument tokens.
/// I.e., the following inputs should give the following outputs,
/// assuming the part before 'word' is: \m
/// c -> c
/// (co -> (, co
/// [color] -> [, color, ]
fn get_escaped_args(word: &str) -> Vec<Token<TroffToken>> {
    let mut v = Vec::new();

    // accepted arg patterns are like:
    // \mc
    // \m(co
    // \m[color]
    // (word contains content after \m, in this example)
    match word.chars().nth(0) {
        Some('(') => {
            let open_tok = Token::new(TroffToken::ArgOpenParen, "(".to_owned(), false);
            v.push(open_tok);

            let arg = &word[1..3];
            let arg_tok = Token::new(TroffToken::CommandArg, arg.to_owned(), false);
            v.push(arg_tok);
        }
        Some('[') => {
            let open_tok = Token::new(TroffToken::ArgOpenBracket, "[".to_owned(), false);
            v.push(open_tok);

            let close_index = word.find(']').unwrap();
            let arg = &word[1..close_index];
            let arg_tok = Token::new(TroffToken::CommandArg, arg.to_owned(), false);
            v.push(arg_tok);

            let close_tok = Token::new(TroffToken::ArgCloseBracket, "]".to_owned(), false);
            v.push(close_tok);
        }
        Some(_) => {
            let arg = &word[0..1];
            let arg_tok = Token::new(TroffToken::CommandArg, arg.to_owned(), false);
            v.push(arg_tok);
        }
        None => {}
    };

    v
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
        let generator = TroffTokenGenerator {};

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
        let generator = TroffTokenGenerator {};

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
        let generator = TroffTokenGenerator {};

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
        let generator = TroffTokenGenerator {};

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
        let word = "\\-lefthalf\\-righthalf\\";
        let generator = TroffTokenGenerator {};

        let actual = generator.generate(word, true);

        let expected = vec![
            Token::new(TroffToken::Backslash, "\\".to_owned(), true),
            Token::new(TroffToken::EscapeCommand, "-".to_owned(), false),
            Token::new(TroffToken::TextWord, "lefthalf".to_owned(), false),
            Token::new(TroffToken::Backslash, "\\".to_owned(), false),
            Token::new(TroffToken::EscapeCommand, "-".to_owned(), false),
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

    #[test]
    fn test_inline_macro() {
        let words = "\\fBHello world\\fP".split_whitespace();
        let generator = TroffTokenGenerator {};

        let mut actual = Vec::new();

        for word in words {
            let mut result = generator.generate(word, true);
            actual.append(&mut result);
        }

        let expected = vec![
            Token::new(TroffToken::Backslash, "\\".to_owned(), true),
            Token::new(TroffToken::EscapeCommand, "f".to_owned(), false),
            Token::new(TroffToken::CommandArg, "B".to_owned(), false),
            Token::new(TroffToken::TextWord, "Hello".to_owned(), false),
            Token::new(TroffToken::TextWord, "world".to_owned(), true),
            Token::new(TroffToken::Backslash, "\\".to_owned(), false),
            Token::new(TroffToken::EscapeCommand, "f".to_owned(), false),
            Token::new(TroffToken::CommandArg, "P".to_owned(), false),
        ];

        assert!(
            actual == expected,
            "expected: {:?}\nactual: {:?}",
            expected,
            actual
        );
    }
}
