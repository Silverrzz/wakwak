#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Abort {
    Yes,
    No,
}

impl From<bool> for Abort {
    #[inline]
    fn from(value: bool) -> Self {
        match value {
            true => Abort::Yes,
            false => Abort::No,
        }
    }
}

#[macro_export]
macro_rules! abort_if {
    ($e:expr) => {
        if $e == Abort::Yes {
            return Abort::Yes;
        }
    };
}
