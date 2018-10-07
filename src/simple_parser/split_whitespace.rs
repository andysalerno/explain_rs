pub trait WhitespaceSplitInclusive {
    fn split_whitespace_inclusive(&self) -> SplitWhitespaceInclusive;
}

impl WhitespaceSplitInclusive for str {
    fn split_whitespace_inclusive(&self) -> SplitWhitespaceInclusive {
        SplitWhitespaceInclusive::new(self)
    }
}

pub struct SplitWhitespaceInclusive<'a> {
    content: &'a str,
    cur_idx: usize,
}

impl<'a> Iterator for SplitWhitespaceInclusive<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_idx >= self.content.len() {
            return None;
        }

        let result_start = self.cur_idx;
        let mut result_end = result_start + 1;
        let mut cur_idx = 0;

        let mut whitespace_mode = false;
        let mut whitespace_char = '\0';

        for c in self.content[result_start..].chars() {
            cur_idx = cur_idx + c.len_utf8();

            if cur_idx == c.len_utf8() {
                if c.is_whitespace() {
                    whitespace_mode = true;
                    whitespace_char = c;
                }
                continue;
            }

            let found_continuation = (c.is_whitespace() && whitespace_mode && c == whitespace_char)
                || (!c.is_whitespace() && !whitespace_mode);

            if found_continuation {
                result_end = result_start + cur_idx;
            } else {
                break;
            }
        }

        self.cur_idx = result_end;

        Some(&self.content[result_start..result_end])
    }
}

impl<'a> SplitWhitespaceInclusive<'a> {
    fn new(content: &'a str) -> Self {
        SplitWhitespaceInclusive {
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

        let result: Vec<&str> = SplitWhitespaceInclusive::new(my_str).collect();

        let expected = vec!["Hello,", " ", "this", " ", "is", " ", "my", " ", "string!"];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_keeps_multilength_spaces() {
        let my_str = "Hello,   this is my    string!";

        let result: Vec<&str> = SplitWhitespaceInclusive::new(my_str).collect();

        let expected = vec![
            "Hello,", "   ", "this", " ", "is", " ", "my", "    ", "string!",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_understands_tabs() {
        let my_str = "Hello,\tthis is my string!";

        let result: Vec<&str> = SplitWhitespaceInclusive::new(my_str).collect();

        let expected = vec!["Hello,", "\t", "this", " ", "is", " ", "my", " ", "string!"];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_understands_newlines() {
        let my_str = "Hello,\nthis is my\n string!";

        let result: Vec<&str> = SplitWhitespaceInclusive::new(my_str).collect();

        let expected = vec![
            "Hello,", "\n", "this", " ", "is", " ", "my", "\n", " ", "string!",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_segregates_whitespaces_types() {
        let my_str = "Hello,\t  \t my whitespace is  \nnon-homogenous!";

        let result: Vec<&str> = SplitWhitespaceInclusive::new(my_str).collect();

        let expected = vec![
            "Hello,",
            "\t",
            "  ",
            "\t",
            " ",
            "my",
            " ",
            "whitespace",
            " ",
            "is",
            "  ",
            "\n",
            "non-homogenous!",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn split_understands_wide_chars() {
        let my_str = "Originally written by Hrvoje Nikšić <hniksic@xemacs.org>.";

        let result: Vec<&str> = SplitWhitespaceInclusive::new(my_str).collect();

        let expected = vec![
            "Originally",
            " ",
            "written",
            " ",
            "by",
            " ",
            "Hrvoje",
            " ",
            "Nikšić",
            " ",
            "<hniksic@xemacs.org>.",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn trait_works() {
        let my_str = "This is a test of the trait!";
        let iterator = my_str.split_whitespace_inclusive();
        let result: Vec<&str> = iterator.collect();

        let expected = vec![
            "This", " ", "is", " ", "a", " ", "test", " ", "of", " ", "the", " ", "trait!",
        ];

        assert_eq!(result, expected);
    }
}
