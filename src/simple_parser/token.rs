// Three simple steps to utilize this to perform tokenization:
// 1. implement Classification for an enum of all your token types
// 2. implement Classifier for your Classification type
// 3. run via tokenizer::tokenize()

// simple unit trait to mark something as a Classification
pub trait TokenClass {}

#[derive(PartialEq, Debug)]
pub struct Token<C: TokenClass> {
    pub class: C,
    pub value: String,
    pub starts_line: bool,
}

impl<C: TokenClass> Token<C> {
    // TODO: value should take &str, and copy as necessary
    pub fn new(class: C, value: String, starts_line: bool) -> Token<C> {
        Token {
            class: class,
            value: value,
            starts_line: starts_line,
        }
    }
}

// trait that defines a TokenGenerator,
// which is given a string word and can
// return the Token.
pub trait TokenGenerator<C: TokenClass> {
    fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<C>>;
    fn is_comment(&self, word: &str) -> bool;
}
