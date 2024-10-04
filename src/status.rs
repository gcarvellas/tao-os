use core::num::TryFromIntError;

#[derive(Debug)]
pub enum ErrorCode {
    InvArg,
    NoMem,
    OutOfBounds,
}

impl From<TryFromIntError> for ErrorCode {
    fn from(_: TryFromIntError) -> Self {
        Self::OutOfBounds
    }
}
