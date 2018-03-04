// implementing tokenization logic for a very small subset of Troff

use simple_parser::token::{Classification, Classifier};

#[derive(Debug)]
pub enum TroffToken {
    Macro,
    TextWord,
}

// TODO: implement #[derive(Classification)] macro...
impl Classification for TroffToken {}

pub struct TroffClassifier;

impl Classifier<TroffToken> for TroffClassifier {
    fn classify(&self, word: &str) -> TroffToken {
        match word {
            w if w.starts_with(".") => TroffToken::Macro,
            _ => TroffToken::TextWord,
        }
    }
}
