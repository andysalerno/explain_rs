use text_format::text_format::TextFormat;

/// Simple struct holding state
/// for font styling: bold, italic, underlined
#[derive(Default)]
pub struct FontStyleState {
    bold: bool,
    italic: bool,
    underlined: bool,
}

#[derive(Copy, Clone)]
pub enum FontStyle {
    Bold,
    Italic,
    Underlined,
    Regular,
}

const SPACE: &str = " ";

impl FontStyleState {
    pub fn set_fontstyle_value(&mut self, s: FontStyle, val: bool) {
        match s {
            FontStyle::Bold => self.bold = val,
            FontStyle::Italic => self.italic = val,
            FontStyle::Underlined => self.underlined = val,

            // Regular is a special case, you can't toggle it.
            // It simply resets everything else to false.
            FontStyle::Regular => {
                self.bold = false;
                self.italic = false;
                self.underlined = false;
            }
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
