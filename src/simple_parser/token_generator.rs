use simple_parser::token::{Token, TokenClass};

pub trait TokenGenerator<C: TokenClass> {
    /// Given a raw string token from some input stream,
    /// returns a vector of Token types generated from the input.
    fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<C>>;

    /// Given a raw string token from some input stream,
    /// return a boolean identifying whether the input begins a comment.
    fn is_comment(&self, word: &str) -> bool;
}
