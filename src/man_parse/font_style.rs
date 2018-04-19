use text_format::text_format::TextFormat;

/// Simple struct holding state
/// for font styling: bold, italic, underlined
#[derive(Default)]
pub struct FontStyleState {
    bold: bool,
    italic: bool,
    underlined: bool,
}

pub enum FontStyle {
    Bold,
    Italic,
    Underlined,
}

const SPACE: &str = " ";

impl FontStyleState {
    pub fn set_fontstyle_value(&mut self, s: FontStyle, val: bool) {
        match s {
            FontStyle::Bold => self.bold = val,
            FontStyle::Italic => self.italic = val,
            FontStyle::Underlined => self.underlined = val,
        }
    }

    pub fn stylize_text(&self, text: &str) -> Option<String> {
        if text == SPACE {
            // don't stylize emtpy space
            return None;
        }

        if self.bold {
            return Some(text.bold());
        }

        if self.italic {
            return Some(text.italic());
        }

        if self.underlined {
            return Some(text.underlined());
        }

        None
    }
}
