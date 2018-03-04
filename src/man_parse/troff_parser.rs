use man_parse::troff_tokenize::TroffToken;
use simple_parser::token::Token;

#[derive(Debug)]
enum ManSection {
    Unknown,
    Name,
    Synopsis,
    Description,
    Options,
}

enum TroffMacros {
    TH,
    RB,
    I,
    SH,
    ll,
    B,
    PP,
    IR,
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

    pub fn parse(&mut self, tokens: I) {
        self.tokens = Some(tokens);

        // consume once to move to the first token
        self.consume();

        while let Some(cur_tok) = self.current_token() {
            println!("parsing token: {:?}", cur_tok);

            if cur_tok.class == TroffToken::Macro && cur_tok.value == ".SH" {
                self.parse_sh();
            } else {
                //self.parse_word();
                self.consume();
            }
        }
    }

    fn parse_sh(&mut self) {
        assert!(self.current_token().unwrap().value == ".SH");
        self.consume_sameline();

        // next token must be a TextWord, which is the SH argument
        self.current_section = match self.current_token() {
            Some(token) if token.value == "NAME" => ManSection::Name,
            Some(token) if token.value == "SYNOPSIS" => ManSection::Synopsis,
            Some(token) if token.value == "DESCRIPTION" => ManSection::Description,
            Some(token) if token.value == "OPTIONS" => ManSection::Options,
            _ => ManSection::Unknown,
        };

        println!("Set section to: {:?}", self.current_section);
    }

    fn parse_word(&mut self) {
        assert!(self.current_token().unwrap().class == TroffToken::TextWord);
        self.consume();
    }

    fn consume(&mut self) {
        self.current_token = self.tokens.as_mut().unwrap().next();
    }

    /// same as consume(), but asserts that the very next token is on the same line
    fn consume_sameline(&mut self) {
        self.consume();

        if let Some(token) = self.current_token {
            assert!(!token.starts_line);
        }
    }

    fn current_token(&self) -> Option<I::Item> {
        self.current_token
    }
}
