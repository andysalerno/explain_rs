pub trait TextFormat {
    fn bold(self) -> String;
    fn italic(self) -> String;
    fn underlined(self) -> String;
}

impl<'a> TextFormat for &'a str {
    fn bold(self) -> String {
        const BOLD_TAG: &str = "\x1b[1m";
        const BOLD_TAG_CLOSE: &str = "\x1b[0m";

        format!("{}{}{}", BOLD_TAG, self, BOLD_TAG_CLOSE)
    }

    fn italic(self) -> String {
        // const italic_tag: &str = "\x1b[3m";
        // const italic_tag_close: &str = "\x1b[0m";

        // format!("{}{}{}", italic_tag, self, italic_tag_close)

        self.underlined()
    }

    fn underlined(self) -> String {
        const UNDERLINE_TAG: &str = "\x1b[4m";
        const UNDERLINE_TAG_CLOSE: &str = "\x1b[0m";

        format!("{}{}{}", UNDERLINE_TAG, self, UNDERLINE_TAG_CLOSE)
    }
}
