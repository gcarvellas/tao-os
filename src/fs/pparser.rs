use core::str::Chars;

use crate::status::ErrorCode;
use alloc::{string::String, vec::Vec};
use config::MAX_PATH;

fn get_drive_by_path(chars: &mut Chars) -> Result<u32, ErrorCode> {
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

fn get_path_part(path: &mut Chars) -> Option<String> {
    let mut res = String::new();

    loop {
        let c = match path.next() {
            None => {
                if res.is_empty() {
                    return None;
                }
                return Some(res);
            },
            Some(c) => c,
        };

        if c == '/' {
            return Some(res);
        }

        if c == '\0' {
            return Some(res);
        }

        res.push(c);
    }
}

pub struct PathRoot {
    drive_no: u32,
    parts: Vec<String>
}

pub fn parse_path(path: String) -> Result<PathRoot, ErrorCode> {
    if path.len() > MAX_PATH {
        return Err(ErrorCode::BadPath);
    }

    let mut tmp_path = path.chars();

    let drive_no = get_drive_by_path(&mut tmp_path)?;

    let mut path_root = PathRoot {
        drive_no,
        parts: Vec::new()
    };

    while let Some(val) = get_path_part(&mut tmp_path) {
        path_root.parts.push(val);
    }

    Ok(path_root)
}
