// Three simple steps to utilize this to perform tokenization:
// 1. implement Classification for an enum of all your token types
// 2. implement Classifier for your Classification type
// 3. run via tokenizer::tokenize()

// simple unit trait to mark something as a Classification
pub trait Classification {}

#[derive(PartialEq, Debug)]
pub struct Token<C: Classification> {
    pub class: C,
    pub value: String,
    pub starts_line: bool,
}

impl<C: Classification> Token<C> {
    pub fn new(class: C, value: String, starts_line: bool) -> Token<C> {
        Token {
            class: class,
            value: value,
            starts_line: starts_line,
        }
    }
}

// trait that defines a Classifier,
// which is given a string word and can
// return the classification of the Token.
pub trait Classifier<C: Classification> {
    fn classify(&self, word: &str) -> C;
}
