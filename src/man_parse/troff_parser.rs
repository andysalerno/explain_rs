use man_parse::font_style::FontStyle;
use man_parse::man_section::ManSection;
use man_parse::troff_term_writer::TroffTermWriter;
use man_parse::troff_tokenize::TroffToken;
use simple_parser::token::Token;
use std;

const SPACE: &str = " ";

// troff is inclusive with indent numbering, but I am not, so this is equal to a troff indent of 8
const DEFAULT_PARAGRAPH_INDENT: usize = 7;

pub struct TroffParser<'a, I>
where
    I: Iterator<Item = &'a Token<TroffToken>>,
{
    tokens: Option<I>,
    current_token: Option<&'a Token<TroffToken>>,
    current_section: Option<ManSection>,

    /// if a section was requested via '-s', store its text here
    // section_text: String,

    /// also a string of section text, but *before* formatting (for debug)
    before_section_text: String,

    /// if a section was requested via '-s', this is the requested section
    parse_section: Option<ManSection>,

    /// functionality for writing output and styling text
    term_writer: TroffTermWriter,
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
            before_section_text: Default::default(),
            parse_section: Default::default(),
            term_writer: TroffTermWriter::new(),
        }
    }

    /// TODO: idiomatically, this should take "self",
    /// and be invoked like "Parser::new().for_section(...)"
    pub fn for_section(section: ManSection) -> Self {
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

        if self.term_writer.is_nofill() && tok.starts_line {
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
                ".IP" => self.parse_ip(),
                ".RS" => self.parse_rs(),
                ".RE" => self.parse_re(),
                ".if" => self.parse_if(),
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
            TroffToken::EmptyLine => self.parse_empty_line(),
            TroffToken::Backslash => self.parse_backslash(),
            TroffToken::Space => self.parse_space(),
            TroffToken::DoubleQuote => self.parse_doublequote(),
            _ => self.parse_textword(),
        }
    }

    /// an empty line in troff generates an empty output line
    fn parse_empty_line(&mut self) {
        self.consume_class(TroffToken::EmptyLine);
        self.add_blank_line();
    }

    /// .P, .PP, or .LP (all mutual aliases)
    /// Adds a full line break.  Also resets indentation and font to initial values.
    fn parse_p(&mut self) {
        self.consume();

        self.term_writer.zero_indent();
        self.term_writer.reset_font_properties();

        self.add_blank_line();
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
                self.consume();
            }
        }

        self.term_writer.set_indent(indent);
    }

    /// .TP [Indent]\n[Label]\n[Paragraph]
    /// Creates a paragraph tagged with a label.
    /// The next input line that contains text is the "label", printed flush-left (to margin).
    /// The line after that is the paragraph text, with "indent" indentation.
    /// If the label is smaller than the indent, the paragraph begins on the same line (not implemented).
    fn parse_tp(&mut self) {
        self.consume_val(".TP");

        // optional argument specifies indentation of paragraph text
        let paragraph_indent = {
            let indent_arg = self.parse_macro_arg().next();
            if indent_arg.is_some() {
                indent_arg.unwrap().value.parse::<usize>().unwrap()
            } else {
                self.stored_or_default_paragraph_indent()
            }
        };

        // output the tag flush-left on a new line
        self.term_writer.zero_indent();
        self.add_linebreak();
        self.consume_spaces();
        self.parse_line();

        // now parse the paragraph line
        self.term_writer.set_indent(paragraph_indent);
        self.term_writer.store_indent();
        self.add_linebreak();
        self.parse_line();
    }

    /// .IP [marker [width]]\n[body]
    /// prints list-style paragraphs.
    /// 'marker' is the tag, like a bullet point \[bu]
    /// 'width' is the indentation of the body from (including) the marker.
    fn parse_ip(&mut self) {
        self.consume_val(".IP");

        // zero indent so tag starts at margin
        self.term_writer.zero_indent();

        // create a blank line before writing the tag
        self.add_blank_line();

        self.consume_spaces();

        // first optional arg is the marker (aka tag), it is printed flush with the margin
        let marker_arg = self.parse_macro_arg();
        for tok in marker_arg {
            self.add_to_output(&tok.value);
            self.add_to_output(SPACE);
        }

        self.consume_spaces();

        // next optional arg is the width to indent for the paragraph
        let indent_tok = self.parse_macro_arg().next();
        let indent_count = if indent_tok.is_some() {
            let f_val = indent_tok.unwrap().value.parse::<f32>().unwrap();
            f_val as usize
        } else {
            self.stored_or_default_paragraph_indent()
        };

        // set indent before printing paragraph
        self.term_writer.set_indent(indent_count);
        self.term_writer.store_indent();

        // start paragraph on newline
        self.add_linebreak();

        // parse the paragraph
        self.parse_line();
    }

    /// Macro: .RS [nnn]
    /// Move the left margin to the right by the value nnn if specified (default unit is ‘n’);
    /// otherwise it is set to the previous indentation value specified with TP, IP, or HP
    /// (or to the default value if none of them have been used yet).
    /// The indentation value is then set to the default.
    /// Calls to the RS macro can be nested.
    /// See: https://www.gnu.org/software/groff/manual/html_node/Man-usage.html
    fn parse_rs(&mut self) {
        self.consume_val(".RS");

        let margin_increase = {
            let indent_arg = self.parse_macro_arg().next();

            if indent_arg.is_some() {
                indent_arg.unwrap().value.parse::<usize>().unwrap()
            } else {
                self.stored_or_default_paragraph_indent()
            }
        };

        self.term_writer.increase_margin(margin_increase);

        // indent value is then reset to default
        self.term_writer.set_indent(DEFAULT_PARAGRAPH_INDENT);
        self.add_linebreak();
    }

    /// decreases the margin by a certain depth
    fn parse_re(&mut self) {
        self.consume_val(".RE");

        let decrease_arg = self.parse_macro_arg().next();

        let pops = if decrease_arg.is_some() {
            decrease_arg.unwrap().value.parse::<usize>().unwrap()
        } else {
            // if no arg provided, just pop once
            1
        };

        for _ in 0..pops {
            self.term_writer.pop_margin();
        }

        self.term_writer.set_indent(DEFAULT_PARAGRAPH_INDENT);
        self.term_writer.store_indent();
    }

    /// Parse the next arg for a macro.
    /// Not used everywhere, but ultimate goal
    /// is to unify arguments under this
    /// since args may be contained in quotes
    /// TODO: can I return a single String, instead of a Vec?
    fn parse_macro_arg(&mut self) -> std::vec::IntoIter<I::Item> {
        self.consume_spaces();

        let empty = Vec::new().into_iter();

        if let Some(tok) = self.current_token() {
            // args are always on the same line
            if tok.starts_line {
                return empty;
            }

            if tok.class == TroffToken::DoubleQuote {
                return self.parse_within_quotes().into_iter();
            } else {
                let result = vec![tok];
                self.consume();
                return result.into_iter();
            }
        } else {
            return empty;
        }
    }

    /// When the current token is a doublequote,
    /// return a vector of every token between
    /// this doublequote and an ending doublequote on the same line.
    /// (returns early if a newline is encountered before a closing doublequote)
    /// (does not include tokens that are Spaces)
    fn parse_within_quotes(&mut self) -> Vec<I::Item> {
        self.consume_class(TroffToken::DoubleQuote);

        let mut result = Vec::new();

        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                return result;
            }

            if tok.class == TroffToken::DoubleQuote {
                self.consume_class(TroffToken::DoubleQuote);
                return result;
            } else if tok.class != TroffToken::Space {
                result.push(tok);
            }

            self.consume();
        }

        result
    }

    /// Sets the rest of the line to bold,
    /// or the very next line if there are no same-line values.
    fn parse_b(&mut self) {
        self.consume_class(TroffToken::Macro);
        self.term_writer.set_fontstyle(FontStyle::Bold);

        self.parse_line();

        self.term_writer.unset_fontstyle(FontStyle::Bold);
    }

    /// Sets the rest of the line to bold,
    /// or the very next line if there are no same-line values.
    fn parse_i(&mut self) {
        self.consume_class(TroffToken::Macro);
        self.term_writer.set_fontstyle(FontStyle::Italic);
        self.parse_line();
        self.term_writer.unset_fontstyle(FontStyle::Italic)
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
                self.term_writer.set_fontstyle(FontStyle::Underlined);
                self.parse_textword();
                self.term_writer.unset_fontstyle(FontStyle::Underlined);
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
                self.term_writer.set_fontstyle(FontStyle::Bold);
                self.parse_word();
                self.term_writer.unset_fontstyle(FontStyle::Bold);
            } else {
                println!("underlined: {:?}", tok);
                self.term_writer.set_fontstyle(FontStyle::Underlined);
                self.parse_word();
                self.term_writer.unset_fontstyle(FontStyle::Underlined);
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
        self.add_linebreak();
        self.term_writer.enable_nofill();
    }

    /// Ends no-fill mode.
    fn parse_fi(&mut self) {
        self.consume();
        self.term_writer.disable_nofill();
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
        self.consume_val("\\");

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

            self.consume();
        }
    }

    fn parse_font_format(&mut self) {
        self.consume_val("f");
        if let Some(tok) = self.current_token() {
            // next arg must be the formatting choice
            match tok.value.as_str() {
                "B" => self.term_writer.set_fontstyle(FontStyle::Bold),
                "I" => self.term_writer.set_fontstyle(FontStyle::Italic),
                "R" | "P" => self.term_writer.reset_font_properties(),
                _ => {}
            }

            self.consume();
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
                "[" => {
                    // consume the '['
                    self.consume_val("[");

                    if let Some(inner_tok) = self.current_token() {
                        if inner_tok.value == "]" {
                            // next token can either close with no arg
                            self.consume_val("]");
                        } else {
                            // or else it is the arg and then closes
                            self.consume();
                            self.consume_val("]");
                        }
                    }
                }
                "(" => {
                    self.consume_val("(");
                    self.consume();
                }
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
        self.consume_val(".SH");

        let sh_args = self.parse_macro_arg();
        let mut arg_str = String::new();

        for arg in sh_args {
            arg_str.push_str(&arg.value);
        }

        self.current_section = match arg_str.as_str() {
            "NAME" => Some(ManSection::Name),
            "SYNOPSIS" => Some(ManSection::Synopsis),
            "DESCRIPTION" => Some(ManSection::Description),
            "OPTIONS" => Some(ManSection::Options),
            _ => Some(ManSection::Unknown),
        };
    }

    /// we aren't smart enough to evaluate expressions
    /// so "if" will simply always be ignored
    fn parse_if(&mut self) {
        self.consume_val(".if");
        self.consume_line();
    }

    /// consume until the beginning of the next line
    fn consume_line(&mut self) {
        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                return;
            }

            self.consume();
        }
    }

    fn parse_doublequote(&mut self) {
        self.consume_class(TroffToken::DoubleQuote);
    }

    fn format_token(token: I::Item) -> String {
        let val = match token.class {
            TroffToken::Space => " ",
            TroffToken::EmptyLine => "[el]",
            _ => &token.value,
        };

        if token.starts_line {
            format!("{}{} ", "\n", val)
        } else {
            format!("{} ", val)
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
        self.term_writer.buf()
    }

    fn section_matches(&self) -> bool {
        self.parse_section.is_some() && self.parse_section == self.current_section
    }

    fn add_blank_line(&mut self) {
        for _ in 0..2 {
            self.add_linebreak();
        }
    }

    fn add_linebreak(&mut self) {
        if self.section_matches() {
            self.term_writer.add_linebreak();
        }
    }

    fn add_to_output(&mut self, s: &str) {
        if self.section_matches() {
            self.term_writer.add_to_buf(s);
        }
    }

    fn add_to_before_output(&mut self, s: &str) {
        if self.section_matches() {
            self.before_section_text.push_str(s);
        }
    }

    fn stored_or_default_paragraph_indent(&self) -> usize {
        match self.term_writer.stored_indent() {
            Some(indent) => indent,
            None => DEFAULT_PARAGRAPH_INDENT,
        }
    }
}
