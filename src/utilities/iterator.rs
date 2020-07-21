pub trait Unique {
    fn all_unique(self) -> bool;
}

impl<T: Clone + Iterator<Item=I>, I: PartialEq> Unique for T {
    fn all_unique(mut self) -> bool {
        // O(n^2), could do with optimisation. Difficult
        // to optimise without a hash set (no heap)
        while let Some(element) = self.next() {
            if self.clone().any(|e| e == element) {
                return false
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn all_unique_in_various_scenarios() {
        assert!( [3, 4, 1, 5].iter().all_unique());
        assert!(![1, 2, 3, 3, 2].iter().all_unique());
        assert!( ["fish", "foot", "fly", "foresight"].iter().all_unique());
        assert!(![None, Some(3), Some(5), None].iter().all_unique());
    }
}
