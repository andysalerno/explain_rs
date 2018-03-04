use man_parse::token::{Classification, Classifier, Token};

pub fn tokenize<C: Classification>(input: &str, classifier: &Classifier<C>) -> Vec<Token<C>> {
    let mut result = Vec::new();

    let split = input.split_whitespace();

    for c in split {
        let class = classifier.classify(&c);
        let token = Token {
            class: class,
            value: c.to_owned(),
        };

        result.push(token);
    }

    result
}

#[cfg(test)]
mod tests {
    use man_parse::token::{Classification, Classifier, Token};
    use man_parse::tokenizer;

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
        fn classify(&self, word: &str) -> TestToken {
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
            Token::new(TestToken::LParen, "(".to_owned()),
            Token::new(TestToken::NumVal(1), "1".to_owned()),
            Token::new(TestToken::AddOp, "+".to_owned()),
            Token::new(TestToken::NumVal(8), "8".to_owned()),
            Token::new(TestToken::RParen, ")".to_owned()),
        ];

        assert!(
            expected == result,
            "expected: {:?} actual: {:?}",
            expected,
            result
        );
    }
}
