use crate::config::MAX_PATH;
use crate::status::ErrorCode;
use core::str::Split;

pub type PathPart<'a> = Split<'a, char>;

fn get_drive_by_path(path: &str) -> Result<u32, ErrorCode> {
    let mut chars = path.chars();

    let drive_no = chars
        .next()
        .ok_or(ErrorCode::BadPath)?
        .to_digit(10)
        .ok_or(ErrorCode::BadPath)?;

    if !chars.take(2).eq([':', '/']) {
        return Err(ErrorCode::BadPath);
    }

    Ok(drive_no)
}

pub struct PathRoot<'a> {
    pub drive_no: u32,
    pub parts: PathPart<'a>,
}

pub fn parse_path(path: &str) -> Result<PathRoot, ErrorCode> {
    if path.len() > MAX_PATH {
        return Err(ErrorCode::BadPath);
    }

    let drive_no = get_drive_by_path(path)?;

    let parts = path[3..].split('/');

    let path_root = PathRoot { drive_no, parts };

    Ok(path_root)
}
