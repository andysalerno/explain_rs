use simple_parser::token::{Classification, Classifier, Token};
use std::str::SplitWhitespace;

pub fn tokenize<C: Classification>(input: &str, classifier: &Classifier<C>) -> Vec<Token<C>> {
    let mut result = Vec::new();

    let split_lines = split_lines(input);

    for line in split_lines {
        for (i, c) in line.enumerate() {
            let starts_line = i == 0;
            let class = classifier.classify(&c, starts_line);
            let token = Token::new(class, c.to_owned(), starts_line);

            result.push(token);
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
    use simple_parser::token::{Classification, Classifier, Token};
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

    impl Classifier<TestToken> for TestClassifier {
        fn classify(&self, word: &str, starts_line: bool) -> TestToken {
            println!("classifying: {}", word);

            match word {
                "(" => return TestToken::LParen,
                ")" => return TestToken::RParen,
                "+" => return TestToken::AddOp,
                w if w.parse::<i32>().is_ok() => TestToken::NumVal(w.parse::<i32>().unwrap()),
                &_ => panic!(format!("found an illegal character: {}", word)),
            }
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
