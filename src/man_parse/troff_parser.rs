use man_parse::troff_tokenize::TroffToken;
use simple_parser::token::Token;
use text_format::text_format::TextFormat;
use man_parse::man_section::ManSection;

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

    /// current state of font styling
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

        while self.current_token().is_some() {
            self.parse_token();
        }
    }

    fn parse_token(&mut self) {
        let tok = if self.current_token().is_some() {
            self.current_token().unwrap()
        } else {
            return;
        };

        if self.font_style.nofill && tok.starts_line {
            self.add_linebreak();
        }

        if tok.class == TroffToken::Macro {
            self.parse_macro();
        } else {
            self.parse_line();
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
                ".TP" => self.parse_tp(),
                ".PD" => self.parse_pd(),
                ".B" => self.parse_b(),
                ".I" => self.parse_i(),
                ".IR" => self.parse_ir(),
                ".BI" => self.parse_bi(),
                ".PP" | ".LP" | ".P" => self.parse_p(),
                _ => {
                    self.add_to_before_output(&format!(
                        "[skipping unknown macro: {:?}]",
                        tok.value
                    ));

                    self.consume();
                }
            }
        }
    }

    /// parse all tokens until the end of the line,
    /// so that the resulting current token is the first of the next line.
    fn parse_line(&mut self) {
        while self.current_token().is_some() {
            self.parse_word();

            if let Some(next_tok) = self.current_token() {
                if next_tok.starts_line {
                    break;
                }
            }
        }
    }

    /// Parse an individual word, which may be a
    /// simple literal string, or an escaped character or command
    fn parse_word(&mut self) {
        let tok = if let Some(tok) = self.current_token() {
            tok
        } else {
            return;
        };

        match tok.class {
            TroffToken::Macro => self.parse_macro(),
            TroffToken::Backslash => self.parse_backslash(),
            TroffToken::Space => self.parse_space(),
            TroffToken::DoubleQuote => self.parse_doublequote(),
            _ => self.parse_textword(),
        }
    }

    /// .P, .PP, or .LP (all mutual aliases)
    /// Adds a full line break.  Also resets indentation and font to initial values.
    fn parse_p(&mut self) {
        self.consume();

        // TODO: just recreate the default FontStyle
        self.font_style.indent = 0;
        self.font_style.bold = false;
        self.font_style.italic = false;
        self.font_style.underlined = false;

        self.add_linebreak();
        self.add_linebreak();
    }

    /// .PD [Distance]
    /// Sets paragraph distance.
    /// If no argument is provided, default to 0.
    /// TODO: I think the implementation is wrong here??
    fn parse_pd(&mut self) {
        self.consume_val(".PD");

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
        self.consume_val(".TP");

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

        self.consume_spaces();

        let temp_indent = self.font_style.indent;
        self.font_style.indent = 0;

        self.add_linebreak();
        self.add_linebreak();

        // next text line is the tag
        self.parse_line();

        self.font_style.indent = temp_indent;

        // now, on the same line, add [space * indent]
        for _ in 0..indent_count {
            self.add_to_output(SPACE);
        }
    }

    /// Sets the rest of the line to bold,
    /// or the very next line if there are no same-line values.
    fn parse_b(&mut self) {
        self.consume();
        self.font_style.bold = true;

        self.parse_line();

        self.font_style.bold = false;
    }

    /// Sets the rest of the line to bold,
    /// or the very next line if there are no same-line values.
    fn parse_i(&mut self) {
        self.consume();
        self.font_style.italic = true;
        self.parse_line();
        self.font_style.italic = false;
    }

    /// Alternates between italic and regular.
    fn parse_ir(&mut self) {
        self.consume_val(".IR");

        let mut italic = true;

        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                break;
            }
            if tok.class == TroffToken::Space {
                self.parse_textword();
                continue;
            }

            if italic {
                self.font_style.underlined = true;
                self.parse_textword();
                self.font_style.underlined = false;
            } else {
                self.parse_textword();
            }

            italic = !italic;
        }
    }

    /// Alternates between bold and italic.
    /// TODO: create "parse_alternating(a: FontStyle, b: FontStyle)"
    fn parse_bi(&mut self) {
        self.consume_val(".BI");

        let mut bold = true;
        let mut in_quote = false;

        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                break;
            }
            if tok.class == TroffToken::Space {
                self.parse_textword();
                if !in_quote {
                    bold = !bold;
                }
                continue;
            }
            if tok.class == TroffToken::Backslash {
                in_quote = !in_quote;
            }

            if bold {
                println!("bold: {:?}", tok);
                self.font_style.bold = true;
                self.parse_word();
                self.font_style.bold = false;
            } else {
                println!("underlined: {:?}", tok);
                self.font_style.underlined = true;
                self.parse_word();
                self.font_style.underlined = false;
            }
        }
    }

    fn parse_textword(&mut self) {
        let cur_tok = self.current_token().unwrap();
        self.add_to_output(&cur_tok.value);
        self.consume();
    }

    fn parse_space(&mut self) {
        self.consume();
        self.add_to_output(SPACE);
    }

    /// Begin no-fill mode, and add a linebreak.
    fn parse_nf(&mut self) {
        self.consume();
        self.add_to_output(LINEBREAK);
        self.font_style.nofill = true;
    }

    /// Ends no-fill mode.
    fn parse_fi(&mut self) {
        self.consume();
        self.font_style.nofill = false;
    }

    /// Adds a linebreak.
    fn parse_br(&mut self) {
        self.consume();
        self.add_linebreak();
    }

    /// Parses a backslash, which escapes some value.
    /// A simple example is '\-', which evaluates to '-'.
    /// A more complicated example is '\fBHello', which
    /// prints 'Hello' in bold.
    fn parse_backslash(&mut self) {
        self.consume();

        if let Some(tok) = self.current_token() {
            match tok.value.as_str() {
                "-" => self.parse_hyphen(),
                "(" => self.parse_special_character(),
                "f" => self.parse_font_format(),
                "m" => self.parse_color_format(),
                _ => self.consume(),
            }
        }
    }

    /// \- gives '-'
    fn parse_hyphen(&mut self) {
        self.consume_val("-");
        self.add_to_output("-");
    }

    /// \(cq gives "'"
    fn parse_special_character(&mut self) {
        self.consume_val("(");
        if let Some(tok) = self.current_token() {
            match tok.value.as_str() {
                "cq" => self.add_to_output("'"),
                // also lq, rq, oq apparently, TODO
                _ => {}
            }
        }
    }

    fn parse_font_format(&mut self) {
        self.consume_val("f");
        if let Some(tok) = self.current_token() {
            // next arg must be the formatting choice
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
        self.consume_val("m");
        if let Some(tok) = self.current_token() {
            // we now have an argument pattern
            // \mc
            // \m(co
            // \m[color]
            match tok.value.as_str() {
                "[" => {}
                _ => self.consume(),
            }
        }
    }

    /// .sp [LineCount]
    /// Adds some amount of spacing lines,
    /// defaulting to two if no arguments
    fn parse_sp(&mut self) {
        self.consume_val(".sp");

        let mut linebreaks = 2;

        if let Some(tok) = self.current_token() {
            if !tok.starts_line {
                // there's an argument to .sp
                if let Ok(parsed) = tok.value.parse::<i32>() {
                    if parsed > 0 {
                        linebreaks += parsed;
                    } else {
                        println!("warning: parsed .sp value < 0, of {}", parsed);
                    }
                }

                self.consume();
            }
        }

        for _ in 0..linebreaks {
            self.add_linebreak();
        }
    }

    /// .SH SubheaderName
    /// example: ".SH options"
    /// This is not implemented correctly, as
    /// you can use quotes to captures spaces and multiple words.
    /// This currently only captures one word, and assumes doublequotes.
    fn parse_sh(&mut self) {
        self.consume();

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

        self.consume();
    }

    fn parse_doublequote(&mut self) {
        self.consume_class(TroffToken::DoubleQuote);
    }

    fn format_token(token: I::Item) -> String {
        if token.starts_line {
            format!("{}{}", LINEBREAK, token.value)
        } else {
            format!(" {}", token.value)
        }
    }

    /// Consume the current token, moving to the next,
    /// and additionally asserting the consumed token has the given class.
    fn consume_class(&mut self, class: TroffToken) {
        assert_eq!(self.current_token().unwrap().class, class);
        self.consume();
    }

    /// Consume the current token, pushing forward the iterator
    /// to the next token.
    fn consume(&mut self) {
        self.current_token = self.tokens.as_mut().unwrap().next();

        if let Some(tok) = self.current_token {
            self.add_to_before_output(&Self::format_token(tok));
        }
    }

    fn consume_val(&mut self, it: &str) {
        assert!(it == self.current_token.unwrap().value);
        self.consume();
    }

    fn consume_spaces(&mut self) {
        while let Some(tok) = self.current_token() {
            if tok.class != TroffToken::Space {
                break;
            }

            self.consume();
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

    /// Add a linebreak to the output,
    /// and also indent from the left based on the current style.
    fn add_linebreak(&mut self) {
        self.add_to_output(LINEBREAK);

        // newlines must receive the current left-margin indent
        for _ in 0..self.font_style.indent {
            self.add_to_output(SPACE);
        }
    }

    fn add_to_output(&mut self, s: &str) {
        if self.parse_section.is_some() && self.parse_section == self.current_section {
            if self.font_style.bold {
                let bold = s.bold();
                self.section_text.push_str(&bold);
            } else if self.font_style.italic {
                let italic = s.underlined();
                self.section_text.push_str(&italic);
            } else if self.font_style.underlined {
                let underlined = s.underlined();
                self.section_text.push_str(&underlined);
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
}
