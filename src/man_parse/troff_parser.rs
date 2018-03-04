use man_parse::troff_tokenize::TroffToken;
use simple_parser::token::Token;

enum ManSection {
    Name,
    Synopsis,
    Description,
}

pub struct TroffParser<'a, I>
where
    I: Iterator<Item = &'a Token<TroffToken>>,
{
    section_name: String,
    section_synopsis: String,
    section_description: String,

    current_section: ManSection,

    tokens: Option<I>,
    current_token: Option<&'a Token<TroffToken>>,
}

impl<'a, I> TroffParser<'a, I>
where
    I: Iterator<Item = &'a Token<TroffToken>>,
{
    pub fn new() -> Self {
        TroffParser {
            section_description: Default::default(),
            section_synopsis: Default::default(),
            section_name: Default::default(),
            current_section: ManSection::Name,
            tokens: None,
            current_token: None,
        }
    }

    pub fn parse(&mut self, mut tokens: I) {
        self.tokens = Some(tokens);
    }

    fn consume(&mut self) {
        self.current_token = self.tokens.as_mut().unwrap().next();
    }

    fn current_token(&self) -> I::Item {
        self.current_token.as_ref().unwrap()
    }
}
