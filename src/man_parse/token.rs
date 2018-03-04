// simple unit trait to mark something as a Classification
pub trait Classification {}

#[derive(PartialEq, Debug)]
pub struct Token<C: Classification> {
    pub class: C,
    pub value: String,
}

impl<C: Classification> Token<C> {
    pub fn new(class: C, value: String) -> Token<C> {
        Token {
            class: class,
            value: value,
        }
    }
}

// trait that defines a Classifier,
// which is given a string word and can
// return the classification of the Token.
pub trait Classifier<C: Classification> {
    fn classify(&self, word: &str) -> C;
}
