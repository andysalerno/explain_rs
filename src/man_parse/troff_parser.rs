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
const SPACE: &str = " ";

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

    /// TODO: idiomatically, this should take "self",
    /// and be invoked like "Parser::new().for_section(...)"
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
            if self.font_style.nofill && cur_tok.starts_line {
                self.add_to_output(LINEBREAK);
            }

            if cur_tok.class == TroffToken::Macro {
                self.parse_macro();
            } else {
                self.parse_line();
            }
        }
    }

    fn parse_macro(&mut self) {
        if let Some(tok) = self.current_token() {
            assert_eq!(tok.class, TroffToken::Macro);
            match tok.value.as_str() {
                ".SH" => self.parse_sh(),
                ".sp" => self.parse_sp(),
                ".br" => self.parse_br(),
                ".nf" => self.parse_nf(),
                ".fi" => self.parse_fi(),
                ".B" => self.parse_b(),
                ".I" => self.parse_i(),
                ".TP" => self.parse_tp(),
                ".PD" => self.parse_pd(),
                ".PP" | ".LP" | ".P" => self.parse_p(),
                _ => {
                    self.add_to_before_output(&format!(
                        "[skipping unknown macro: {:?}]",
                        tok.value
                    ));

                    self.consume();
                    self.parse_line();
                }
            }
        }
    }

    /// parse all tokens until the end of the line,
    /// so that the resulting current token is the first of the next line.
    fn parse_line(&mut self) {
        while let Some(tok) = self.current_token() {
            // TODO: performance -- branch prediction might
            // not like alternating space/textword so much
            // maybe unify both if it comes down to it
            if tok.class == TroffToken::Backslash {
                self.parse_backslash();
            } else if tok.class == TroffToken::Space {
                self.parse_space();
            } else if tok.class == TroffToken::DoubleQuote {
                self.parse_doublequote();
            } else {
                self.parse_textword();
            }

            if let Some(next_tok) = self.current_token() {
                if next_tok.starts_line {
                    break;
                }
            }
        }
    }

    fn parse_p(&mut self) {
        self.consume();

        self.add_to_output(LINEBREAK);
        self.add_to_output(LINEBREAK);
    }

    /// sets the indent value
    /// if no argument is provided
    /// default to 0.
    fn parse_pd(&mut self) {
        self.consume_it(".PD");

        let mut indent = 0;

        if let Some(tok) = self.current_token() {
            if !tok.starts_line {
                indent = tok.value.parse::<usize>().unwrap();
            }

            self.consume();
        }

        self.font_style.indent = indent;
    }

    /// .TP [Indent]
    /// The next input line that contains text is the "tag".
    /// The tag is printed at the normal indent, and then on the same line
    /// the remaining text is given at the [Index] distance.  If the tag is
    /// larger than the [Indent], the text begins on the next line.
    /// If no [Indent] is provided, use the default or the previous one.
    fn parse_tp(&mut self) {
        self.consume_it(".TP");

        let cur_tok = if let Some(tok) = self.current_token() {
            tok
        } else {
            return;
        };

        let indent_count = if !cur_tok.starts_line {
            let parsed_indent = cur_tok.value.parse::<usize>().unwrap();
            self.font_style.indent = parsed_indent;

            // consume the optional TP indent argument
            self.consume();

            parsed_indent
        } else {
            self.font_style.indent
        };

        // next text line is the tag
        self.parse_line();

        // now, on the same line, add [space * indent]
        for _ in 0..indent_count {
            self.add_to_output(SPACE);
        }
    }

    fn parse_b(&mut self) {
        self.consume();
        self.font_style.bold = true;

        self.parse_line();

        self.font_style.bold = false;
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
        // assert_eq!(
        //     cur_tok.class,
        //     TroffToken::TextWord,
        //     "saw tok: {:?}",
        //     cur_tok.value
        // );
        self.add_to_output(&cur_tok.value);
        self.consume();
    }

    fn parse_space(&mut self) {
        self.add_to_output(" ");
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

    fn consume(&mut self) {
        self.current_token = self.tokens.as_mut().unwrap().next();

        if let Some(tok) = self.current_token {
            self.add_to_before_output(&Self::format_token(tok));
        }
    }

    /// Consume until the current token starts a new line
    fn consume_line(&mut self) {
        self.consume();

        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                break;
            }

            self.consume();
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
