#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pair<T> {
    One(T),
    Two(T, T),
}

pub struct PairIterator<I>
where
    I: Iterator + ?Sized,
{
    inner: I,
}

pub trait IntoPair: Iterator {
    fn pairs(self) -> PairIterator<Self>;
}

impl<I: Iterator> IntoPair for I {
    fn pairs(self) -> PairIterator<Self> {
        PairIterator { inner: self }
    }
}

impl<T, I: Iterator<Item = T>> Iterator for PairIterator<I> {
    type Item = Pair<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let left = self.inner.next()?;
        match self.inner.next() {
            None => Some(Pair::One(left)),
            Some(right) => Some(Pair::Two(left, right)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pairs_iterator_yields_correct_values() {
        let mut numbers = [1, 2, 3, 4, 5].iter().pairs();
        assert_eq!(Some(Pair::Two(&1, &2)), numbers.next());
        assert_eq!(Some(Pair::Two(&3, &4)), numbers.next());
        assert_eq!(Some(Pair::One(&5)), numbers.next());
        assert_eq!(None, numbers.next());

        let mut empty = [0i32; 0].iter().pairs();
        assert_eq!(None, empty.next());

        let mut single = [10].iter().pairs();
        assert_eq!(Some(Pair::One(&10)), single.next());
    }
}
