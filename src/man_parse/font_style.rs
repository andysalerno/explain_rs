#[derive(Default)]
pub struct FontStyle {
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,

    /// is no-fill mode active?
    /// prints lines "as-is", including whitespace.
    /// enabled with macro ".nf", disabled with ".fi"
    pub nofill: bool,

    // The three values defining line layout
    // see: https://www.gnu.org/software/groff/manual/html_node/Line-Layout.html
    /// The indentation distance from the left margin
    indent: usize,
    prev_indent: Option<usize>,

    /// The distance from the left of the page where text can begin.
    /// AKA the "left margin" or "page offset" location
    margin: usize,

    /// The maximum length in characters of a line, before it is wrapped.
    pub line_length: usize,

    /// stack to track indentation scopes
    margin_stack: Vec<usize>,
}

const DEFAULT_INDENT: usize = 5;
const DEFAULT_MARGIN_INCREASE: usize = 5;

impl FontStyle {
    pub fn reset_font_properties(&mut self) {
        self.bold = false;
        self.italic = false;
        self.underlined = false;
    }

    pub fn set_indent(&mut self, indent: usize) {
        //self.prev_indent = Some(self.indent);
        self.indent = indent;
    }

    pub fn indent(&self) -> usize {
        self.indent
    }

    pub fn prev_indent(&self) -> usize {
        match self.prev_indent {
            Some(v) => v,
            None => DEFAULT_INDENT,
        }
    }

    // pub fn use_prev_indent(&mut self) {
    //     self.indent = self.prev_indent();
    //     // TODO: should we set prev_indent to indent first?
    //     // Should it be a stack of value history?
    // }

    /// Sets the indentation to 0.
    pub fn zero_indent(&mut self) {
        self.indent = 0;
    }

    pub fn margin(&self) -> usize {
        self.margin
    }

    pub fn increase_margin(&mut self, size: usize) {
        self.margin += size;
        self.margin_stack.push(size);
    }

    pub fn increase_margin_default(&mut self) {
        self.margin += DEFAULT_MARGIN_INCREASE;
        self.margin_stack.push(DEFAULT_INDENT);
    }

    pub fn pop_margin(&mut self) {
        // TODO: unwrap safety
        let amount = self.margin_stack.pop().unwrap();
        self.margin -= amount;
    }

    /// Get the text starting position in a new line.
    /// This is simply the margin + indent.
    /// Most text should start after this many spaces on a line.
    pub fn text_start_pos(&self) -> usize {
        self.margin + self.indent
    }
}
