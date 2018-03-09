// implementing tokenization logic for a very small subset of Troff

use simple_parser::token::{Classification, Token, TokenGenerator};

#[derive(Debug, PartialEq)]
pub enum TroffToken {
    Macro,
    TextWord,
    DoubleQuote,
    Backslash,
}

// TODO: implement #[derive(Classification)] macro...
impl Classification for TroffToken {}

pub struct TroffClassifier;

impl TokenGenerator<TroffToken> for TroffClassifier {
    fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<TroffToken>> {
        let mut tokens = Vec::new();

        if starts_line && word.starts_with('.') {
            let tok = Token::new(TroffToken::Macro, word.to_owned(), true);
            tokens.push(tok);
            return tokens;
        }

        let mut base_index: usize = 0;
        for (walker, c) in word.chars().enumerate() {
            let special_tok = match c {
                '\\' => Some(Token::new(
                    TroffToken::Backslash,
                    "\\".to_owned(),
                    starts_line,
                )),
                '"' => Some(Token::new(
                    TroffToken::DoubleQuote,
                    "\"".to_owned(),
                    starts_line,
                )),
                _ => None,
            };

            if special_tok.is_some() {
                if walker > base_index + 1 {
                    let prev_word = word[base_index..walker].to_owned();
                    let prev_tok = Token::new(TroffToken::TextWord, prev_word, starts_line);
                    tokens.push(prev_tok);
                }

                tokens.push(special_tok.unwrap());

                base_index = walker + 1;
            }
        }

        if base_index == 0 && word.len() > 0 {
            // we never found anything, so push the whole word
            let word_tok = Token::new(TroffToken::TextWord, word.to_owned(), starts_line);
            tokens.push(word_tok);
        }

        tokens
    }

    fn is_comment(&self, word: &str) -> bool {
        word.starts_with("\\\"") || word.starts_with(".\\\"") || word.starts_with("\\#")
            || word == "."
    }
}

// impl TokenGenerator<TroffToken> for TroffClassifier {
//     fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<TroffToken>> {
//         let mut result = Vec::new();

//         let open_quote = match word.starts_with("\"") {
//             true => Some(Token::new(
//                 TroffToken::DoubleQuote,
//                 "\"".to_owned(),
//                 starts_line,
//             )),
//             false => None,
//         };

//         let close_quote = match word.ends_with("\"") && word.len() > 1 {
//             true => Some(Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false)),
//             false => None,
//         };

//         let mut trimmed_word = match open_quote {
//             Some(_) => &word[1..],
//             None => word,
//         };

//         trimmed_word = match close_quote {
//             Some(_) => &trimmed_word[..trimmed_word.len() - 1],
//             None => trimmed_word,
//         };

//         // push open quote, if available
//         if open_quote.is_some() {
//             let tok = Token::new(TroffToken::DoubleQuote, "\"".to_owned(), starts_line);
//             result.push(tok);
//         }

//         // push word
//         {
//             let word_category = match word.starts_with(".") {
//                 true => TroffToken::Macro,
//                 false => TroffToken::TextWord,
//             };

//             let word_starts_line = open_quote.is_none() && starts_line;
//             let word_tok = Token::new(word_category, trimmed_word.to_owned(), word_starts_line);
//             result.push(word_tok);
//         }

//         // push close quote, if available
//         if close_quote.is_some() {
//             let tok = Token::new(TroffToken::DoubleQuote, "\"".to_owned(), false);
//             result.push(tok);
//         }

//         result
//     }

// }
