#[derive(Debug)]
pub enum ErrorCode {
    InvArg,
    NoMem,
    Io,
    BadPath,
    DiskNotUs,
    FsNotUs,
    RdOnly,
    NoFdAvailable,
    NotFound,
    NoFs,
}
