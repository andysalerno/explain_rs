#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ManSection {
    Unknown,
    Name,
    Synopsis,
    Description,
    Options,
}

impl<'a> From<&'a str> for ManSection {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "name" => ManSection::Name,
            "synopsis" => ManSection::Synopsis,
            "options" => ManSection::Options,
            "description" => ManSection::Description,
            _ => ManSection::Unknown,
        }
    }
}
