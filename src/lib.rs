use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

struct Sources<'a> {
    current: Option<&'a (dyn error::Error + 'static)>,
}
impl<'a> Sources<'a> {
    fn new(current: Option<&'a (dyn error::Error + 'static)>) -> Sources<'a> {
        Sources { current }
    }
}
impl<'a> Iterator for Sources<'a> {
    type Item = &'a (dyn error::Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        self.current = current.and_then(error::Error::source);

        current
    }
}

pub struct Report<'a> {
    error: &'a (dyn error::Error + 'static),
}

impl<'a> Report<'a> {
    //   type E = &'a (dyn error::Error + 'static);

    pub fn new(e: &'a (dyn error::Error + 'static)) -> Self {
        Report { error: e }
    }

    fn sources(&self) -> Sources<'a> {
        Sources::new(self.error.source())
    }
}

impl<'a> Display for Report<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;

        for source in self.sources() {
            write!(f, ": {}", source)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Display, Formatter, Result};

    #[derive(Debug)]
    struct TestErrorCauseCause {
        msg: &'static str,
    }

    impl TestErrorCauseCause {
        fn new(msg: &'static str) -> TestErrorCauseCause {
            TestErrorCauseCause { msg }
        }
    }

    impl Display for TestErrorCauseCause {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self.msg)
        }
    }

    impl error::Error for TestErrorCauseCause {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            //Some(&self.cause);
            None
        }
    }

    #[derive(Debug)]
    struct TestErrorCause {
        msg: &'static str,
        cause: TestErrorCauseCause,
    }

    impl TestErrorCause {
        fn new(msg: &'static str, cause: TestErrorCauseCause) -> TestErrorCause {
            TestErrorCause { msg, cause }
        }
    }

    impl Display for TestErrorCause {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self.msg)
        }
    }

    impl error::Error for TestErrorCause {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            Some(&self.cause)
        }
    }

    #[derive(Debug)]
    struct TestError {
        msg: &'static str,
        cause: TestErrorCause,
    }

    impl TestError {
        fn new(msg: &'static str, cause: TestErrorCause) -> TestError {
            TestError { msg, cause }
        }
    }

    impl Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self.msg)
        }
    }

    impl error::Error for TestError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            Some(&self.cause)
        }
    }

    static EEE: &str = "test EEE";
    static EE: &str = "test EE";
    static E: &str = "test E";

    #[test]
    fn sources_works() {
        let eee = TestErrorCauseCause::new(EEE);
        let ee = TestErrorCause::new(EE, eee);
        let e = TestError::new(E, ee);
        let sources = super::Sources::new(Some(&e));
        let mut i: u8 = 0;
        for o in sources {
            match i {
                0 => assert_eq!(o.to_string(), E),
                1 => assert_eq!(o.to_string(), EE),
                2 => assert_eq!(o.to_string(), EEE),
                _ => panic!("Unexpected variant: {}", i),
            };
            i += 1;
        }
        assert_eq!(i, 3);
        let r = super::Report::new(&e);
        assert_eq!(r.to_string(), format!("{E}: {EE}: {EEE}"))
    }
}
