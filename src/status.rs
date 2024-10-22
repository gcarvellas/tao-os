use core::num::TryFromIntError;

#[derive(Debug)]
pub enum ErrorCode {
    InvArg,
    NoMem,
    OutOfBounds,
    Io,
    BadPath,
    DiskNotUs,
    FsNotUs,
    Busy,
    RdOnly,
    NoFdAvailable,
    NotFound,
    NoFs,
}

impl From<TryFromIntError> for ErrorCode {
    fn from(_: TryFromIntError) -> Self {
        Self::OutOfBounds
    }
}
