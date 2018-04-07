#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn is_one(&self) -> bool {
        match *self {
            OneOrMany::One(_) => true,
            _ => false,
        }
    }

    pub fn is_many(&self) -> bool {
        !self.is_one()
    }
}

#[cfg(test)]
mod tests {
    use super::OneOrMany;

    #[test]
    fn test_is_one() {
        let one = OneOrMany::One(42);

        assert!(one.is_one());
    }

    #[test]
    fn test_is_many() {
        let many = OneOrMany::Many(vec![1, 2, 3, 4]);

        assert!(many.is_many());
    }
}
