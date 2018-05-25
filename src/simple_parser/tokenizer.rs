use simple_parser::split_whitespace::WhitespaceSplitInclusive;
use simple_parser::token::{Token, TokenClass};
use simple_parser::token_generator::TokenGenerator;

const EMPTY: &str = "";

pub fn tokenize<C: TokenClass>(input: &str, classifier: &TokenGenerator<C>) -> Vec<Token<C>> {
    let mut result = Vec::new();

    for line in input.lines() {
        // special case where the line is a totally blank line
        if line.len() == 0 {
            let mut tokens = classifier.generate(EMPTY, true);
            result.append(&mut tokens);
            continue;
        }

        let mut word_iter = line.split_whitespace_inclusive().enumerate();
        while let Some((i, word)) = word_iter.next() {
            let starts_line = i == 0;

            if starts_line && classifier.is_comment(&word) {
                break;
            }

            // a single word might generate multiple tokens
            let mut tokens = classifier.generate(&word, starts_line);

            result.append(&mut tokens);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use simple_parser::token::{Token, TokenClass};
    use simple_parser::token_generator::TokenGenerator;
    use simple_parser::tokenizer;

    #[derive(PartialEq, Debug)]
    enum TestToken {
        LParen,
        RParen,
        NumVal(i32),
        AddOp,
        Space,
    }

    impl TokenClass for TestToken {}

    struct TestGenerator;

    impl TokenGenerator<TestToken> for TestGenerator {
        fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<TestToken>> {
            match word {
                "(" => return vec![Token::new(TestToken::LParen, "(".into(), starts_line)],
                ")" => return vec![Token::new(TestToken::RParen, ")".into(), starts_line)],
                "+" => return vec![Token::new(TestToken::AddOp, "+".into(), starts_line)],
                w if w.parse::<i32>().is_ok() => vec![
                    Token::new(
                        TestToken::NumVal(w.parse::<i32>().unwrap()),
                        w.into(),
                        starts_line,
                    ),
                ],
                _ => vec![Token::new(TestToken::Space, word.into(), starts_line)],
            }
        }

        fn is_comment(&self, _word: &str) -> bool {
            false
        }
    }

    #[test]
    fn test_simple_line() {
        let s = "( 1 +      8 )";

        let generator = TestGenerator {};
        let result = tokenizer::tokenize(s, &generator);

        let expected = vec![
            Token::new(TestToken::LParen, "(".to_owned(), true),
            Token::new(TestToken::Space, " ".to_owned(), false),
            Token::new(TestToken::NumVal(1), "1".to_owned(), false),
            Token::new(TestToken::Space, " ".to_owned(), false),
            Token::new(TestToken::AddOp, "+".to_owned(), false),
            Token::new(TestToken::Space, "      ".to_owned(), false),
            Token::new(TestToken::NumVal(8), "8".to_owned(), false),
            Token::new(TestToken::Space, " ".to_owned(), false),
            Token::new(TestToken::RParen, ")".to_owned(), false),
        ];

        assert_eq!(
            expected, result,
            "expected: {:?} actual: {:?}",
            expected, result
        );
    }
}
