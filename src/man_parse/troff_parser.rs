use man_parse::troff_tokenize::TroffToken;
use simple_parser::token::Token;

#[derive(Debug, PartialEq)]
pub enum ManSection {
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
    tokens: Option<I>,
    current_token: Option<&'a Token<TroffToken>>,
    current_section: Option<ManSection>,

    /// if a section was requested via '-s', store its text here
    section_text: String,

    /// if a section was requested via '-s', this is the requested section
    parse_section: Option<ManSection>,
}

impl<'a, I> TroffParser<'a, I>
where
    I: Iterator<Item = &'a Token<TroffToken>>,
{
    pub fn new() -> Self {
        TroffParser {
            section_text: Default::default(),
            parse_section: None,
            current_section: None,
            tokens: None,
            current_token: None,
        }
    }

    pub fn for_section(section: ManSection) -> Self {
        println!("creating parser for section: {:?}", section);
        let mut p = Self::new();
        p.parse_section = Some(section);
        p
    }

    pub fn parse(&mut self, tokens: I) {
        self.tokens = Some(tokens);

        // consume once to move to the first token
        self.consume();

        while let Some(cur_tok) = self.current_token() {
            let cur_tok_val = Self::format_token(&self.current_token().unwrap());
            self.add_to_output(&cur_tok_val);

            if cur_tok.class == TroffToken::Macro {
                match cur_tok.value.as_str() {
                    ".SH" => self.parse_sh(),
                    ".sp" => self.parse_spacing(),
                    _ => self.consume(),
                }
            } else {
                self.consume();
            }
        }
    }

    fn add_to_output(&mut self, s: &str) {
        if self.parse_section.is_some() && self.parse_section == self.current_section {
            self.section_text.push_str(s);
        }
    }

    fn parse_fontescape(&mut self) {}

    fn parse_spacing(&mut self) {
        assert!(self.current_token().unwrap().value == ".sp");

        self.consume();

        let tok = self.current_token();
        if tok.is_none() || tok.unwrap().starts_line {
            self.add_to_output("\n");
            return;
        }

        let val: i32 = tok.unwrap().value.parse().unwrap();

        for i in 0..val {
            self.add_to_output("\n");
        }
    }

    fn parse_sh(&mut self) {
        assert!(
            self.current_token().unwrap().value == ".SH",
            "found: {}",
            self.current_token().unwrap().value
        );

        self.consume_sameline();

        if self.current_token.unwrap().class == TroffToken::DoubleQuote {
            self.parse_doublequote();
        }

        println!(
            "current tok: {}\nclass: {:?}",
            self.current_token().unwrap().value,
            self.current_token().unwrap().class
        );

        // next token must be a TextWord, which is the SH argument
        self.current_section = match self.current_token() {
            Some(token) if token.value == "NAME" => Some(ManSection::Name),
            Some(token) if token.value == "SYNOPSIS" => Some(ManSection::Synopsis),
            Some(token) if token.value == "DESCRIPTION" => Some(ManSection::Description),
            Some(token) if token.value == "OPTIONS" => Some(ManSection::Options),
            _ => Some(ManSection::Unknown),
        };

        self.parse_word();

        println!("Set section to: {:?}", self.current_section);
    }

    fn format_token(token: I::Item) -> String {
        if token.starts_line {
            format!("\n{}", token.value)
        } else {
            format!(" {}", token.value)
        }
    }

    fn parse_word(&mut self) {
        assert!(self.current_token().unwrap().class == TroffToken::TextWord);
        self.consume();
    }

    fn parse_doublequote(&mut self) {
        assert!(self.current_token().unwrap().class == TroffToken::DoubleQuote);
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

    pub fn section_text(&self) -> &str {
        &self.section_text
    }
}
