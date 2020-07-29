use core::fmt::{Display, Debug};

/// Only way to define an error type is as a static reference to a
/// StaticError type. This is the trait that should be used in
/// interfaces.
pub trait Error: private::Sealed {
    fn source(&self) -> Option<&'static dyn StaticError>;
}

impl<T: StaticError> private::Sealed for &'static T {}
impl<T: StaticError> Error for &'static T {
    fn source(&self) -> Option<&'static dyn StaticError> {
        (*self).source()
    }
}

/// Implement this trait on your custom error type, and return
/// a static reference to it to supply the Error trait.
pub trait StaticError: Display + Debug
{
    fn source(&self) -> Option<&'static dyn StaticError>;
}

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::{Formatter, self};

    trait GenericInterface {
        type Error: Error;
        fn attempt() -> Result<(), Self::Error>;
    }

    #[derive(Debug)]
    enum TestError {
        BadThing,
        BigBadThing(&'static str),
    }

    struct ConcreteImplementer {}

    impl GenericInterface for ConcreteImplementer {
        type Error = &'static TestError;
        fn attempt() -> Result<(), Self::Error> {
            // Small, so we can just inline
            const ERR: TestError = TestError::BadThing;
            Err(&ERR)
        }
    }



    impl Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                TestError::BadThing => write!(f, "Bad Thing happened"),
                TestError::BigBadThing(what) => write!(f, "Big Bad Thing happened: {}", what),
            }
        }
    }

    impl StaticError for TestError {
        fn source(&self) -> Option<&'static dyn StaticError> {
            None
        }
    }


    #[test]
    fn what() {
        assert!(false)
    }
}
