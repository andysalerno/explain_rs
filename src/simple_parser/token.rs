// Simple unit trait to mark something as a TokenClass.
// Implement this trait on an enum of your token types.
pub trait TokenClass {}

#[derive(PartialEq, Debug)]
pub struct Token<C: TokenClass> {
    pub class: C,
    pub value: String,
    pub starts_line: bool,
}

impl<T: TokenClass> Token<T> {
    // TODO: value should take &str, and copy as necessary
    pub fn new(class: T, value: String, starts_line: bool) -> Token<T> {
        Token {
            class: class,
            value: value,
            starts_line: starts_line,
        }
    }
}
