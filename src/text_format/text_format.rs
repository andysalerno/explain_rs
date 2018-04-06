pub trait TextFormat {
    fn bold(self) -> String;
    fn italic(self) -> String;
    fn underlined(self) -> String;
}

impl<'a> TextFormat for &'a str {
    fn bold(self) -> String {
        const bold_tag: &str = "\x1b[1m";
        const bold_tag_close: &str = "\x1b[0m";

        format!("{}{}{}", bold_tag, self, bold_tag_close)
    }

    fn italic(self) -> String {
        // const italic_tag: &str = "\x1b[3m";
        // const italic_tag_close: &str = "\x1b[0m";

        // format!("{}{}{}", italic_tag, self, italic_tag_close)

        self.underlined()
    }

    fn underlined(self) -> String {
        const underline_tag: &str = "\x1b[4m";
        const underline_tag_close: &str = "\x1b[0m";

        format!("{}{}{}", underline_tag, self, underline_tag_close)
    }
}
