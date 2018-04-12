pub struct TermWriter {
    buf: String,
}

impl TermWriter {
    pub fn add_to_buf(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    pub fn print_buf(&self) {
        println!("{}", self.buf);
    }
}
