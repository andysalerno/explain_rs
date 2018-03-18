// implementing tokenization logic for a very small subset of Troff

use simple_parser::token::{Token, TokenClass, TokenGenerator};

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

        let mut peek_iter = word.chars().enumerate();

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

                // after a '\' is always an escaped character
                if special_class == TroffToken::Backslash {
                    if let Some(next) = peek_iter.next() {
                        let next_index = next.0;
                        let next_char = next.1;

                        let escaped =
                            Token::new(TroffToken::EscapeCommand, next_char.to_string(), false);

                        tokens.push(escaped);
                        base_index = next_index + 1;

                        // after the escaped token, we might now have arguments
                        // i.e., '\fB' takes arg 'B'
                        let args = get_args(next_char, &word[next_index + 1..]);

                        if args.len() > 0 {
                            let args_tok =
                                Token::new(TroffToken::CommandArg, args.to_string(), false);

                            tokens.push(args_tok);

                            for _ in 0..args.len() {
                                peek_iter.next();
                            }

                            base_index += args.len();
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

        tokens.push(Token::new(TroffToken::Space, " ".to_string(), false));

        tokens
    }

    fn is_comment(&self, word: &str) -> bool {
        word.starts_with("\\\"") || word.starts_with(".\\\"") || word.starts_with("\\#")
            || word == "."
    }
}

/// Some escape commands can have args,
/// like '\fB' where the escape command is 'f'
/// and the arg is 'B'.
///
/// Given a command char, and the slice immediately following it,
/// return which subset of the slice is made up of args.
///
/// i.e., given command 'f' and word 'BHello',
/// the result should be 'B'.
fn get_args(command_char: char, word: &str) -> &str {
    match command_char {
        'f' => &word[0..1],

        // these commands take no args, so always empty slice
        '-' => &word[0..0],
        _ => &word[0..0],
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
        let word = "\\-lefthalf\\-righthalf\\";
        let generator = TroffClassifier {};

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
        let generator = TroffClassifier {};

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
