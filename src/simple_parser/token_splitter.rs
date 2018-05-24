pub struct TokenSplitter<'a> {
    content: &'a str,
    cur_idx: usize,
}

impl<'a> Iterator for TokenSplitter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_idx >= self.content.len() {
            return None;
        }

        let result_start = self.cur_idx;
        let mut result_end = result_start + 1;

        let mut whitespace_mode = false;

        for (index, c) in self.content[result_start..].chars().enumerate() {
            if index == 0 {
                whitespace_mode = c.is_whitespace();
                continue;
            }

            let found_continuation =
                (c.is_whitespace() && whitespace_mode) || (!c.is_whitespace() && !whitespace_mode);

            if found_continuation {
                result_end = result_start + index + 1;
            } else {
                break;
            }
        }

        self.cur_idx = result_end;

        Some(&self.content[result_start..result_end])
    }
}

impl<'a> TokenSplitter<'a> {
    fn new(content: &'a str) -> Self {
        TokenSplitter {
            content,
            cur_idx: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_includes_whitespace() {
        let my_str = "Hello, this is my string!";

        let tok_iter = TokenSplitter::new(my_str);

        let result: Vec<&str> = tok_iter.collect();

        let expected = vec!["Hello,", " ", "this", " ", "is", " ", "my", " ", "string!"];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_keeps_multilength_spaces() {
        let my_str = "Hello,   this is my    string!";

        let tok_iter = TokenSplitter::new(my_str);

        let result: Vec<&str> = tok_iter.collect();

        let expected = vec![
            "Hello,", "   ", "this", " ", "is", " ", "my", "    ", "string!"
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_understands_tabs() {
        let my_str = "Hello,\tthis is my string!";

        let tok_iter = TokenSplitter::new(my_str);

        let result: Vec<&str> = tok_iter.collect();

        let expected = vec!["Hello,", "\t", "this", " ", "is", " ", "my", " ", "string!"];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_understands_newlines() {
        let my_str = "Hello,\nthis is my\n string!";

        let tok_iter = TokenSplitter::new(my_str);

        let result: Vec<&str> = tok_iter.collect();

        let expected = vec![
            "Hello,", "\n", "this", " ", "is", " ", "my", "\n ", "string!"
        ];

        assert_eq!(result, expected);
    }
}
