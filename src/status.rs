use core::num::TryFromIntError;

#[derive(Debug)]
pub enum ErrorCode {
    InvArg,
    NoMem,
    OutOfBounds,
    Io,
}

impl From<TryFromIntError> for ErrorCode {
    fn from(_: TryFromIntError) -> Self {
        Self::OutOfBounds
    }
}
