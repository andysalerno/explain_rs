use simple_parser::preprocessor::Preprocessor;

pub struct TroffPreprocessor;

impl Preprocessor for TroffPreprocessor {
    fn preprocess(input: &str) -> String {
        "hello".into()
    }
}
