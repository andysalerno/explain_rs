extern crate term_size;
use man_parse::font_style::{FontStyle, FontStyleState};
use std::cmp;

// TODO: change this name, or move to parser -- writer should not care about paragraph
// troff is inclusive with indent numbering, but I am not, so this is equal to a troff indent of 8
const DEFAULT_PARAGRAPH_INDENT: usize = 7;
const DEFAULT_MARGIN_INCREASE: usize = 5;

const DEFAULT_LINE_LENGTH: usize = 80;
const RIGHT_MARGIN_LENGTH: usize = 8;
const MIN_LINE_LENGTH: usize = 80;

const LINEBREAK: &str = "\n";
const SPACE: &str = " ";

fn term_width() -> usize {
    let width = term_size::dimensions()
        .unwrap_or((DEFAULT_LINE_LENGTH, 0))
        .0;

    // if their term width is below a certain amount,
    // we have a cutoff so our output remains sane.
    cmp::max(width, MIN_LINE_LENGTH) - RIGHT_MARGIN_LENGTH
}

/// A Troff-knowledgeable terminal writer.
/// Handles indentation and styling.
#[derive(Default)]
pub struct TroffTermWriter {
    /// italic, bold, underlined
    font_style: FontStyleState,

    /// is no-fill mode active?
    /// prints lines "as-is", including whitespace.
    /// enabled with macro ".nf", disabled with ".fi"
    nofill: bool,

    // The three values defining line layout
    // see: https://www.gnu.org/software/groff/manual/html_node/Line-Layout.html
    /// The indentation distance from the left margin
    indent: usize,

    /// History of scoped indentations
    prev_indent: Option<usize>,

    /// The distance from the left of the page where text can begi.
    /// AKA the "left margin" or "page offset" location
    margin: usize,

    /// The maximum length in characters of a line, before it is wrapped.
    /// Margin and indent count towards this limit.
    line_length: usize,

    /// stack to track indentation scopes
    /// TODO: redundant with prev_indent?
    margin_stack: Vec<usize>,

    /// A string buffer that owners of this struct
    /// can append to, and ultimately flush to terminal
    output_buf: String,

    /// The length of the current line being written.
    /// When this exceeds line_length, we will wrap.
    cur_line_len: usize,
}

impl TroffTermWriter {
    pub fn new() -> Self {
        let mut tw: Self = Default::default();
        tw.line_length = term_width();

        tw
    }

    /// Clear bold/italic/underlined properties
    pub fn reset_font_properties(&mut self) {
        self.font_style = Default::default();
    }

    pub fn indent(&self) -> usize {
        self.indent
    }

    /// Sets the indentation to 0.
    pub fn zero_indent(&mut self) {
        self.indent = 0;
    }

    pub fn prev_indent(&self) -> usize {
        match self.prev_indent {
            Some(v) => v,
            None => DEFAULT_PARAGRAPH_INDENT,
        }
    }

    pub fn store_prev_indent(&mut self) {
        self.prev_indent = Some(self.indent);
    }

    pub fn margin(&self) -> usize {
        self.margin
    }

    /// Increase the margin by some count of characters.
    /// The margin is how many chars from the left until the indent begins.
    pub fn increase_margin(&mut self, size: usize) {
        self.margin += size;
        self.margin_stack.push(size);
    }

    /// Increase the margin by some default amount.
    /// Some Troff macros do not specify an increase count, so they use this.
    pub fn increase_margin_default(&mut self) {
        self.increase_margin(DEFAULT_MARGIN_INCREASE);
    }

    /// Pop away the last indent increase,
    /// and decrease the margin by that amount to undo it.
    pub fn pop_margin(&mut self) {
        // TODO: unwrap safety
        let amount = self.margin_stack.pop().unwrap();
        self.margin -= amount;
    }

    pub fn enable_nofill(&mut self) {
        self.nofill = true;
    }

    pub fn disable_nofill(&mut self) {
        self.nofill = false;
    }

    pub fn is_nofill(&self) -> bool {
        self.nofill
    }

    /// Enables a FontStyle, such as Bold
    pub fn set_fontstyle(&mut self, s: FontStyle) {
        self.set_fontstyle_value(s, true);
    }

    /// Disables a FontStyle, such as Bold
    pub fn unset_fontstyle(&mut self, s: FontStyle) {
        self.set_fontstyle_value(s, false);
    }

    fn set_fontstyle_value(&mut self, s: FontStyle, val: bool) {
        self.font_style.set_fontstyle_value(s, val);
    }

    /// Add some text to the output buffer, inserting linebreaks
    /// if the given text exceeds the limit.
    /// It's expected that text contains no linebreaks on its own.
    pub fn add_to_buf(&mut self, text: &str) {
        if text == "\n" {
            self.add_linebreak();
            return;
        }

        // TODO: need to not count zero-width chars (and count >1 width chars?)
        if self.cur_line_len + text.len() > self.line_length {
            self.add_linebreak();

            if text == SPACE {
                return;
            }
        }

        self.cur_line_len += text.len();

        if let Some(stylized) = self.font_style.stylize_text(text) {
            self.output_buf.push_str(&stylized);
        } else {
            self.output_buf.push_str(text);
        }
    }

    pub fn buf(&self) -> &str {
        &self.output_buf
    }

    /// Set the indent to be used when adding lines
    /// (line breaks will also respect the indent)
    pub fn set_indent(&mut self, count: usize) {
        self.prev_indent = Some(self.indent);
        self.indent = count;
    }

    pub fn add_linebreak(&mut self) {
        self.output_buf.push_str(LINEBREAK);

        self.cur_line_len = 0;

        for _ in 0..self.text_start_pos() {
            self.add_to_buf(SPACE);
        }
    }

    fn text_start_pos(&self) -> usize {
        self.margin + self.indent
    }
}
