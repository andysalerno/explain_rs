pub trait TextFormat {
    fn bold(self) -> String;
    // fn italic(self) -> Self;
    // fn underlined(self) -> Self;
}

impl<'a> TextFormat for &'a str {
    fn bold(self) -> String {
        const bold_tag: &str = "\x1b[1m";
        const bold_tag_close: &str = "\x1b[0m";

        format!("{}{}{}", bold_tag, self, bold_tag_close)
    }
}
