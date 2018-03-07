use simple_parser::token::{Classification, Token, TokenGenerator};
use std::str::SplitWhitespace;

pub fn tokenize<C: Classification>(input: &str, classifier: &TokenGenerator<C>) -> Vec<Token<C>> {
    let mut result = Vec::new();

    let split_lines = split_lines(input);

    for line in split_lines {
        for (i, mut word) in line.enumerate() {
            let starts_line = i == 0;
            if starts_line && classifier.is_comment(&word) {
                break;
            }

            // if word.starts_with("\"") {
            //     word = &word[1..];
            //     let quote_token = Token::new()
            // }

            let mut tokens = classifier.generate(&word, starts_line);

            result.append(&mut tokens);
        }
    }

    result
}

fn split_lines(input: &str) -> Vec<SplitWhitespace> {
    let mut result = Vec::new();

    let lines = input.lines();

    for line in lines {
        result.push(line.split_whitespace());
    }

    result
}

#[cfg(test)]
mod tests {
    use simple_parser::token::{Classification, Token, TokenGenerator};
    use simple_parser::tokenizer;

    #[derive(PartialEq, Debug)]
    enum TestToken {
        LParen,
        RParen,
        NumVal(i32),
        AddOp,
    }

    impl Classification for TestToken {}

    struct TestClassifier;

    impl TokenGenerator<TestToken> for TestClassifier {
        fn generate(&self, word: &str, starts_line: bool) -> Vec<Token<TestToken>> {
            println!("classifying: {}", word);

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
                &_ => panic!(format!("found an illegal character: {}", word)),
            }
        }

        fn is_comment(&self, word: &str) -> bool {
            false
        }
    }

    #[test]
    fn test_tokenize() {
        let s = "( 1 +      8 )";

        let classifier = TestClassifier {};
        let result = tokenizer::tokenize(s, &classifier);

        let expected = vec![
            Token::new(TestToken::LParen, "(".to_owned(), true),
            Token::new(TestToken::NumVal(1), "1".to_owned(), false),
            Token::new(TestToken::AddOp, "+".to_owned(), false),
            Token::new(TestToken::NumVal(8), "8".to_owned(), false),
            Token::new(TestToken::RParen, ")".to_owned(), false),
        ];

        assert!(
            expected == result,
            "expected: {:?} actual: {:?}",
            expected,
            result
        );
    }
}
