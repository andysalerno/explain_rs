// implementing tokenization logic for a very small subset of Troff

use simple_parser::token::{Classification, Classifier};

#[derive(Debug, PartialEq)]
pub enum TroffToken {
    Macro,
    TextWord,
}

// TODO: implement #[derive(Classification)] macro...
impl Classification for TroffToken {}

pub struct TroffClassifier;

impl Classifier<TroffToken> for TroffClassifier {
    fn classify(&self, word: &str, starts_line: bool) -> TroffToken {
        match word {
            w if w.starts_with(".") && starts_line => TroffToken::Macro,
            _ => TroffToken::TextWord,
        }
    }

    fn is_comment(&self, word: &str) -> bool {
        word.starts_with("\\\"") || word.starts_with(".\\\"") || word.starts_with("\\#")
            || word == "."
    }
}
