pub trait Contains<T: PartialEq> {
    fn contains(&self, x: &T) -> bool;
}

impl<T: PartialEq> Contains<T> for Option<T> {
    fn contains(&self, x: &T) -> bool {
        match self {
            Some(y) => y == x,
            None => false,
        }
    }
}
