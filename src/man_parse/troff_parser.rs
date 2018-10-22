use man_parse::man_section::ManSection;
use man_parse::term_writer::font_style::FontStyle;
use man_parse::term_writer::troff_term_writer::TroffTermWriter;
use man_parse::troff_token_generator::TroffToken;
use simple_parser::token::Token;

const SPACE: &str = " ";

pub struct TroffParser<'a, I>
where
    // TODO: I shouldn't be the iterator, it should be a trait,
    // and Token should be a trait, not a class (I think)
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

    /// The arguments to search for in the man page, if any.
    args: Option<Vec<String>>,

    debug: bool,
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
            args: Default::default(),
            debug: false,
        }
    }

    pub fn enable_debug(&mut self) {
        self.debug = true;
        self.term_writer.enable_debug();
    }

    pub fn for_section(mut self, section: ManSection) -> Self {
        self.parse_section = Some(section);
        self
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = Some(args);
        self
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
            match tok.class {
                TroffToken::TextWord | TroffToken::Whitespace => self.add_linebreak_single(),
                _ => {}
            }
        }

        if tok.class == TroffToken::Macro {
            // TODO: parse_line already handles parse_macro,
            // so maybe this distinction isn't necessary
            self.parse_macro();
        } else {
            self.parse_line();
            if !self.term_writer.is_curline_whitespace_only() {
                self.add_to_output(SPACE);
            }
        }
    }

    fn parse_macro(&mut self) {
        if let Some(tok) = self.current_token() {
            assert_eq!(tok.class, TroffToken::Macro);
            match tok.value.as_str() {
                ".SH" => self.parse_sh(),
                ".SS" => self.parse_ss(),
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
                ".RB" => self.parse_rb(),
                ".RE" => self.parse_re(),
                ".if" => self.parse_if(),
                ".PP" | ".LP" | ".P" => self.parse_p(),
                _ => {
                    // TODO: remove this, uneeded
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
            TroffToken::Whitespace => self.parse_whitespace(),
            TroffToken::DoubleQuote => self.parse_doublequote(),
            _ => self.parse_textword(),
        }
    }

    /// Parse a literal textword, as-is.
    /// Use parse_word() to parse macros, escapes, etc.
    fn parse_textword(&mut self) {
        let cur_tok = self.current_token().unwrap();

        self.add_to_output(&cur_tok.value);
        self.consume();

        // TODO: this instead
        // self.consume_class(TroffToken::TextWord);
    }

    /// Parses an empty line, which parses as a blank line.
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

        if !self.term_writer.is_curline_whitespace_only() {
            self.add_blank_line();
        }
    }

    /// .PD [Distance]
    /// Sets paragraph distance.
    /// If no argument is provided, default to 0.
    fn parse_pd(&mut self) {
        self.consume_val(".PD");

        let mut indent = 0;

        if let Some(tok) = self.current_token() {
            if !tok.starts_line && tok
                .value
                .chars()
                .filter(|t| !t.is_whitespace())
                .next()
                .is_some()
            {
                match tok.value.parse::<usize>() {
                    Ok(num) => indent = num,
                    Err(s) => panic!("{}, tok: {}", s, tok.value),
                }
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
            let indent_arg = self.parse_macro_arg().into_iter().next();
            if let Some(arg) = indent_arg {
                arg.value.parse::<usize>().unwrap()
            } else {
                self.term_writer.stored_or_default_indent()
            }
        };

        // output the tag flush-left on a new line
        self.term_writer.zero_indent();
        self.add_linebreak_single();
        self.consume_spaces();
        self.parse_line();

        // now parse the paragraph line
        self.term_writer.set_indent(paragraph_indent);
        self.term_writer.store_indent();
        self.add_linebreak_single();
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
            // todo: we should be calling parse_textword on these somehow...
            self.add_to_output(&tok.value);
        }

        self.consume_spaces();

        // next optional arg is the width to indent for the paragraph
        let indent_tok = self.parse_macro_arg().into_iter().next();
        if let Some(tok) = indent_tok {
            let f_val = tok.value.parse::<f32>().unwrap_or(0f32);
            self.term_writer.set_indent(f_val as usize);
            self.term_writer.store_indent();
        } else {
            let indent = self.term_writer.stored_or_default_indent();
            self.term_writer.set_indent(indent);
        }

        // start paragraph on newline
        self.add_linebreak_single();

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
            let indent_arg = self.parse_macro_arg().into_iter().next();

            if let Some(arg) = indent_arg {
                arg.value.parse::<usize>().unwrap()
            } else {
                self.term_writer.stored_or_default_indent()
            }
        };

        self.term_writer.increase_margin(margin_increase);

        // indent value is then reset to 0
        self.term_writer.set_indent(0);
        self.add_linebreak_single();
    }

    /// decreases the margin by a certain depth
    fn parse_re(&mut self) {
        self.consume_val(".RE");

        let decrease_arg = self.parse_macro_arg().into_iter().next();

        let mut pops = if let Some(tok) = decrease_arg {
            tok.value.parse::<usize>().unwrap()
        } else {
            // if no arg provided, just pop once
            1
        };

        for _ in 0..pops {
            self.term_writer.pop_margin();
        }

        self.term_writer.zero_indent();
        self.term_writer.clear_stored_indent();
    }

    /// Parse the next arg for a macro.
    /// Note that a single arg can span multiple whitespaces,
    /// if wrapped in a quote like "this is one arg".  It would be four without the quotes.
    fn parse_macro_arg(&mut self) -> Vec<I::Item> {
        self.consume_spaces();

        let empty = Vec::new();

        if let Some(tok) = self.current_token() {
            // args are always on the same line
            if tok.starts_line {
                return empty;
            }

            if tok.class == TroffToken::DoubleQuote {
                return self.get_within_quotes();
            } else {
                let result = vec![tok];
                self.consume();
                return result;
            }
        } else {
            return empty;
        }
    }

    /// When the current token is a doublequote,
    /// return a vector of every token between
    /// this doublequote and an ending doublequote on the same line.
    /// (returns early if a newline is encountered before a closing doublequote)
    fn get_within_quotes(&mut self) -> Vec<I::Item> {
        self.consume_class(TroffToken::DoubleQuote);

        let mut result = Vec::new();

        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                return result;
            }

            if tok.class == TroffToken::DoubleQuote {
                self.consume_class(TroffToken::DoubleQuote);
                return result;
            } else {
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
        self.parse_line_with_style(FontStyle::Bold);
    }

    /// Sets the rest of the line to bold,
    /// or the very next line if there are no same-line values.
    fn parse_i(&mut self) {
        self.consume_class(TroffToken::Macro);
        self.parse_line_with_style(FontStyle::Italic);
    }

    fn parse_line_with_style(&mut self, style: FontStyle) {
        self.term_writer.set_fontstyle(style);
        self.consume_spaces();
        self.parse_line();
        self.term_writer.unset_fontstyle(style);
        self.add_to_output(SPACE);
    }

    /// Alternates between italic and regular.
    fn parse_ir(&mut self) {
        self.consume_val(".IR");
        self.parse_alternation(FontStyle::Italic, FontStyle::Regular);
    }

    /// Alternates between regular and bold.
    fn parse_rb(&mut self) {
        self.consume_val(".RB");
        self.parse_alternation(FontStyle::Regular, FontStyle::Bold);
    }

    /// Generic logic for parsing a macro that alternates between two font styles.
    /// Examples:
    /// .RB alternates between regular and bold
    /// .IR alternates between italic and regular.
    fn parse_alternation(&mut self, first: FontStyle, second: FontStyle) {
        let mut on_first = true;

        self.consume_spaces();

        while let Some(tok) = self.current_token() {
            if tok.starts_line {
                // these macros only operate on one line.
                self.add_to_output(SPACE);
                return;
            }
            if tok.class == TroffToken::Whitespace {
                // only on whitespace do we alternate between styles
                on_first = !on_first;
                self.consume_spaces();
                continue;
            }

            let cur_fontstyle = if on_first { first } else { second };

            self.term_writer.set_fontstyle(cur_fontstyle);

            if tok.class == TroffToken::DoubleQuote {
                // quotes can group together tokens that will all have the same styling
                let tok_group = self.get_within_quotes();
                for t in tok_group {
                    // todo: probably have to skip spaces
                    self.add_to_output(&t.value);
                }
            } else {
                // otherwise, we just parse a single word
                self.parse_word();
            }
            self.term_writer.unset_fontstyle(cur_fontstyle);
        }
    }

    /// Alternates between bold and italic.
    fn parse_bi(&mut self) {
        self.consume_val(".BI");
        self.parse_alternation(FontStyle::Bold, FontStyle::Italic);
    }

    fn parse_whitespace(&mut self) {
        let space_tok = self.current_token().unwrap();
        self.add_to_output(&space_tok.value);

        self.consume();
    }

    /// Begin no-fill mode, and add a linebreak.
    fn parse_nf(&mut self) {
        self.consume();
        self.add_linebreak_single();
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
        self.add_linebreak_single();
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
                "f" => {
                    self.parse_font_format();

                    if let Some(next_tok) = self.current_token() {
                        if next_tok.starts_line {
                            self.add_to_output(SPACE);
                        }
                    }
                }
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
    /// either from an optional argument or a default amount.
    fn parse_sp(&mut self) {
        self.consume_val(".sp");

        let mut linebreaks = 2;

        let arg = self.parse_macro_arg();
        if arg.len() > 0 {
            if let Ok(parsed) = arg[0].value.parse::<i32>() {
                if parsed > 0 {
                    linebreaks = parsed;
                }
            }
        }

        if !self.term_writer.is_curline_whitespace_only() {
            for _ in 0..linebreaks {
                self.add_linebreak();
            }
        }
    }

    /// .SH SubheaderName
    /// example: ".SH OPTIONS"
    /// or
    /// ".SH\nOPTIONS"
    fn parse_sh(&mut self) {
        self.consume_val(".SH");

        let mut arg_str = String::new();

        loop {
            let arg = self.parse_macro_arg();
            if arg.len() == 0 {
                break;
            }

            arg.into_iter().for_each(|a| arg_str.push_str(&a.value));
        }

        if self.parse_section.is_some() {
            self.current_section = match arg_str.as_str() {
                "NAME" => Some(ManSection::Name),
                "SYNOPSIS" => Some(ManSection::Synopsis),
                "DESCRIPTION" => Some(ManSection::Description),
                "OPTIONS" => Some(ManSection::Options),
                _ => Some(ManSection::Unknown),
            };
        } else {
            // output the subheader with zero indent in bold
            self.term_writer.zero_margin();
            self.term_writer.zero_indent();

            if !self.term_writer.is_curline_whitespace_only() {
                self.add_blank_line();
            } else {
                self.add_linebreak();
            }

            self.term_writer.set_fontstyle(FontStyle::Bold);
            self.add_to_output(&arg_str);
            self.term_writer.reset_font_properties();

            self.term_writer.default_margin();
            self.term_writer.zero_indent();
            self.add_linebreak_single();
        }
    }

    /// Parse "sub section" macro
    /// Similar to "sub header" .SH,
    /// except doesn't print flush-left.
    fn parse_ss(&mut self) {
        self.consume_val(".SS");

        //self.term_writer.zero_indent();
        self.add_blank_line();

        let arg = self.parse_macro_arg();

        self.term_writer.set_fontstyle(FontStyle::Bold);
        for tok in arg {
            self.add_to_output(&tok.value);
        }
        self.term_writer.reset_font_properties();
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
            TroffToken::Whitespace => " ",
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
            if tok.class != TroffToken::Whitespace {
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

    pub fn result_text(&self) -> &str {
        self.term_writer.buf()
    }

    fn section_matches(&self) -> bool {
        self.parse_section.is_none() || self.parse_section == self.current_section
    }

    /// Add a single completely blank line (i.e. 2 newlines).
    fn add_blank_line(&mut self) {
        for _ in 0..2 {
            self.add_linebreak();
        }
    }

    /// Adds a linebreak, but only if the current line has length > 0.
    /// Some macros, such as .sp or .br, can never result in more than
    /// one blank line in a row.  I.e., the following:
    /// Hello
    /// .br
    /// .br
    /// world
    ///
    /// Must result in this output:
    /// Hello
    /// world
    fn add_linebreak_single(&mut self) {
        if !self.term_writer.is_curline_whitespace_only() {
            // if there was already a line break, we don't add another...
            self.add_linebreak();
        } else {
            // ... however, if the indent/margin has changed, we do want to add that.
            self.term_writer.set_whitespace_to_startpos();
        }
    }

    /// Add a single linebreak (i.e. a newline \\n)
    fn add_linebreak(&mut self) {
        if self.section_matches() {
            self.term_writer.add_linebreak();
        }
    }

    fn add_to_output(&mut self, s: &str) {
        // this might have to be where we do all the work
        // if s.starts_with("-") && self.term_writer.is_curline_whitespace_only() {
        //     println!("found this: {}", s);
        // }

        if self.section_matches() {
            self.term_writer.add_to_buf(s);
        }
    }

    fn add_to_before_output(&mut self, s: &str) {
        if self.section_matches() {
            self.before_section_text.push_str(s);
        }
    }
}
