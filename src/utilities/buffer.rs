pub trait CollectSlice: Iterator {
    /// Collects an iterator into a given slice, returning the number of collected items.
    fn collect_slice(&mut self, slice: &mut [Self::Item]) -> usize;
}

pub trait TryCollectSlice: Iterator {
    type Element;
    type Error;

    /// Attempts to collect an iterator into a given slice, returning the number of collected items.
    fn try_collect_slice(&mut self, slice: &mut [Self::Element]) -> Result<usize, Self::Error>;
}

impl<I: Iterator> CollectSlice for I {
    fn collect_slice(&mut self, slice: &mut [Self::Item]) -> usize {
        slice.iter_mut().zip(self).fold(0, |count, (dest, item)| {
            *dest = item;
            count + 1
        })
    }
}

impl<I, T, E> TryCollectSlice for I
where
    I: Iterator<Item = Result<T, E>>,
{
    type Element = T;
    type Error = E;
    fn try_collect_slice(&mut self, slice: &mut [Self::Element]) -> Result<usize, Self::Error> {
        slice.iter_mut().zip(self).try_fold(0, |count, (dest, item)| {
            *dest = item?;
            Ok(count + 1)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn collecting_various_types_in_slices() {
        const ELEMENTS: usize = 10;
        let mut ints = [0usize; ELEMENTS];
        assert_eq!(ELEMENTS, (0..ELEMENTS).collect_slice(&mut ints));
        assert_eq!(5, ints[5]);

        let mut letters = ['a'; ELEMENTS];
        assert_eq!(3, (0..3u8).map(|i| ('a' as u8 + i) as char).collect_slice(&mut letters));
        assert_eq!('c', letters[2]);
    }

    #[test]
    fn collecting_fallibly() {
        const ELEMENTS: usize = 10;
        let mut ints = [0u8; ELEMENTS];
        let to_collect: [Result<u8, ()>; 3] = [Ok(3), Ok(2), Err(())];
        assert!(to_collect.iter().copied().try_collect_slice(&mut ints).is_err());

        let to_collect: [Result<u8, ()>; 3] = [Ok(3), Ok(2), Ok(1)];
        assert_eq!(Ok(3), to_collect.iter().copied().try_collect_slice(&mut ints));
    }
}
