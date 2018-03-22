use man_parse::troff_tokenize::TroffToken;
use simple_parser::token::Token;
use text_format::text_format::TextFormat;

#[derive(Debug, PartialEq)]
pub enum ManSection {
    Unknown,
    Name,
    Synopsis,
    Description,
    Options,
}

#[derive(Default)]
struct FontStyle {
    bold: bool,
    italic: bool,
    underlined: bool,

    /// is no-fill mode active?
    /// prints lines "as-is", including whitespace.
    /// enabled with macro ".nf", disabled with ".fi"
    nofill: bool,

    /// The current text indent
    indent: usize,
}

impl<'a> From<&'a str> for ManSection {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "name" => ManSection::Name,
            "synopsis" => ManSection::Synopsis,
            "options" => ManSection::Options,
            "description" => ManSection::Description,
            _ => ManSection::Unknown,
        }
    }
}

const LINEBREAK: &str = "\n";

pub struct TroffParser<'a, I>
where
    I: Iterator<Item = &'a Token<TroffToken>>,
{
    tokens: Option<I>,
    current_token: Option<&'a Token<TroffToken>>,
    current_section: Option<ManSection>,

    /// if a section was requested via '-s', store its text here
    section_text: String,

    /// also a string of section text, but *before* formatting (for debug)
    before_section_text: String,

    /// if a section was requested via '-s', this is the requested section
    parse_section: Option<ManSection>,

    font_style: FontStyle,
}

impl<'a, I> TroffParser<'a, I>
where
    I: Iterator<Item = &'a Token<TroffToken>>,
{
    pub fn new() -> Self {
        TroffParser {
            tokens: Default::default(),
            current_token: Default::default(),
            current_section: Default::default(),
            section_text: Default::default(),
            before_section_text: Default::default(),
            parse_section: Default::default(),
            font_style: Default::default(),
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
            // TODO: remove cur_tok_val, the method, the next line
            // let cur_tok_val = Self::format_token(cur_tok);
            // self.add_to_before_output(&cur_tok_val);

            if self.font_style.nofill && cur_tok.starts_line {
                self.add_to_output(LINEBREAK);
            }

            if cur_tok.class == TroffToken::Macro {
                match cur_tok.value.as_str() {
                    ".SH" => self.parse_sh(),
                    ".sp" => self.parse_sp(),
                    ".br" => self.parse_br(),
                    ".nf" => self.parse_nf(),
                    ".fi" => self.parse_fi(),
                    ".B" => self.parse_b(),
                    ".I" => self.parse_i(),
                    ".PP" | ".LP" | ".P" => self.parse_p(),
                    //_ => self.consume(),
                    _ => {
                        println!("[skipping unknown macro: {:?}", cur_tok.value);
                        self.parse_remaining_line();
                    }
                }
            } else if cur_tok.class == TroffToken::Backslash {
                self.parse_backslash();
            } else if cur_tok.class == TroffToken::TextWord {
                self.parse_textword();
            } else if cur_tok.class == TroffToken::Space {
                self.parse_space();
            } else {
                self.consume();
            }
        }
    }

    fn parse_p(&mut self) {
        self.consume();

        self.add_to_output(LINEBREAK);
        self.add_to_output(LINEBREAK);
    }

    /// .TP [Indent]
    /// The next input line that contains text is the "tag".
    /// The tag is printed at the normal indent, and then on the same line
    /// the remaining text is given at the [Index] distance.  If the tag is
    /// larger than the [Indent], the text begins on the next line.
    /// If no [Indent] is provided, use the default or the previous one.
    fn parse_tp(&mut self) {
        self.consume_it(".TP");

        if self.current_token().is_none() {
            return;
        }

        let cur_tok = self.current_token().unwrap();

        let indent_count = if !cur_tok.starts_line {
            let parsed_indent = cur_tok.value.parse::<usize>().unwrap();
            self.font_style.indent = parsed_indent;

            parsed_indent
        } else {
            self.font_style.indent
        };
    }

    fn parse_b(&mut self) {
        self.consume();
        self.font_style.bold = true;

        self.parse_remaining_line();

        self.font_style.bold = false;

        // while let Some(cur_tok) = self.current_token() {
        //     if cur_tok.starts_line {
        //         break;
        //     }

        //     if cur_tok.class == TroffToken::Backslash {
        //         self.parse_backslash();
        //     } else {
        //         self.add_to_output(&cur_tok.value.bold());

        //         self.consume();
        //     }
        // }
    }

    fn parse_i(&mut self) {
        self.consume();

        while let Some(cur_tok) = self.current_token() {
            if cur_tok.starts_line {
                break;
            }

            self.add_to_output(&cur_tok.value.italic());

            self.consume();
        }
    }

    fn parse_textword(&mut self) {
        let cur_tok = self.current_token().unwrap();
        self.add_to_output(&cur_tok.value);
        self.consume();
    }

    fn parse_nf(&mut self) {
        // nofill mode also adds a linebreak
        self.add_to_output(LINEBREAK);
        self.font_style.nofill = true;
        self.consume();
    }

    fn parse_fi(&mut self) {
        self.font_style.nofill = false;
        self.consume();
    }

    fn parse_br(&mut self) {
        self.add_to_output(LINEBREAK);
        self.consume();
    }

    fn parse_space(&mut self) {
        self.add_to_output(" ");
        self.consume();
    }

    fn add_to_output(&mut self, s: &str) {
        if self.parse_section.is_some() && self.parse_section == self.current_section {
            if self.font_style.bold {
                let bold = s.bold();
                self.section_text.push_str(&bold);
            } else {
                self.section_text.push_str(s);
            }
        }
    }

    fn add_to_before_output(&mut self, s: &str) {
        if self.parse_section.is_some() && self.parse_section == self.current_section {
            self.before_section_text.push_str(s);
        }
    }

    fn parse_backslash(&mut self) {
        self.consume();

        if self.current_token().is_none() {
            return;
        }

        let cur_tok = self.current_token().unwrap();

        match cur_tok.value.as_str() {
            "-" => self.parse_hyphen(),
            "(" => self.parse_special_character(),
            "f" => self.parse_font_format(),
            "m" => self.parse_color_format(),
            _ => self.consume(),
        }
    }

    fn parse_hyphen(&mut self) {
        self.consume_it("-");
        self.add_to_output("-");
    }

    fn parse_special_character(&mut self) {
        self.consume_it("(");

        if let Some(tok) = self.current_token() {
            match tok.value.as_str() {
                "cq" => self.add_to_output("'"),
                _ => {}
            }
        }
    }

    fn parse_font_format(&mut self) {
        self.consume_it("f");

        // next arg must be the formatting choice
        if let Some(tok) = self.current_token() {
            match tok.value.as_str() {
                "B" => {
                    self.font_style.bold = true;
                    self.consume();
                }
                "R" | "P" => {
                    self.font_style.bold = false;
                    self.consume();
                }
                _ => self.consume(),
            }
        }
    }

    fn parse_color_format(&mut self) {
        self.consume_it("m");

        // we now have an argument pattern
        // \mc
        // \m(co
        // \m[color]
        if let Some(tok) = self.current_token() {
            match tok.value.as_str() {
                "[" => {}
                _ => self.consume(),
            }
        }
    }

    fn parse_sp(&mut self) {
        self.consume_it(".sp");

        let mut linebreaks = 2;

        if let Some(tok) = self.current_token() {
            if !tok.starts_line {
                // there's an argument to .sp
                println!("parsing .sp arg: {}", tok.value);

                if let Ok(parsed) = tok.value.parse::<i32>() {
                    if parsed > 0 {
                        linebreaks += parsed;
                    }
                }

                self.consume();
            }
        }

        for _ in 0..linebreaks {
            self.add_to_output(LINEBREAK);
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

        if let Some(cur_tok) = self.current_token() {
            self.current_section = match cur_tok.value.as_str() {
                "NAME" => Some(ManSection::Name),
                "SYNOPSIS" => Some(ManSection::Synopsis),
                "DESCRIPTION" => Some(ManSection::Description),
                "OPTIONS" => Some(ManSection::Options),
                _ => Some(ManSection::Unknown),
            };
        }

        self.parse_word();
    }

    fn format_token(token: I::Item) -> String {
        if token.starts_line {
            format!("{}{}", LINEBREAK, token.value)
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

    /// consume all tokens until the end of the line,
    /// so that the resulting current token is the first of the next line.
    /// Assumes first token is a Macro that will get consumed without any consideration.
    fn parse_remaining_line(&mut self) {
        self.consume();

        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                break;
            }

            if tok.class == TroffToken::Backslash {
                self.parse_backslash();
            } else {
                self.add_to_output(&tok.value);
                self.consume();
            }
        }
    }

    fn consume(&mut self) {
        self.current_token = self.tokens.as_mut().unwrap().next();

        if let Some(tok) = self.current_token {
            self.add_to_before_output(&Self::format_token(tok));
        }
    }

    fn consume_it(&mut self, it: &str) {
        assert!(it == self.current_token.unwrap().value);
        self.consume();
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

    pub fn before_section_text(&self) -> &str {
        &self.before_section_text
    }

    pub fn section_text(&self) -> &str {
        &self.section_text
    }
}
