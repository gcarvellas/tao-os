use core::{cell::RefCell, str::Chars};

use crate::status::ErrorCode;
use alloc::{rc::Rc, string::String};
use config::MAX_PATH;

type PathPart = Rc<RefCell<PathPartInner>>;

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
            None => return None,
            Some(c) => c,
        };

        if c == '/' {
            path.next();
            return Some(res);
        }

        if c == '\0' {
            return Some(res);
        }

        res.push(c);
    }
}

fn parse_path_part(mut last_part: Option<PathPart>, chars: &mut Chars) -> Option<PathPart> {
    let path_part = get_path_part(chars)?;

    let part = Rc::new(RefCell::new(PathPartInner {
        part: path_part,
        next: None,
    }));

    if let Some(ref mut last_part) = last_part {
        last_part.borrow_mut().next = Some(Rc::clone(&part));
    }

    Some(part)
}

pub struct PathRoot {
    drive_no: u32,
    first: PathPart,
}

struct PathPartInner {
    part: String,
    next: Option<PathPart>,
}

pub fn parse_path(path: String) -> Result<PathRoot, ErrorCode> {
    if path.len() > MAX_PATH {
        return Err(ErrorCode::BadPath);
    }

    let mut tmp_path = path.chars();

    let drive_no = get_drive_by_path(&mut tmp_path)?;

    let first_part = parse_path_part(None, &mut tmp_path).ok_or(ErrorCode::BadPath)?;

    let path_root = PathRoot {
        drive_no,
        first: Rc::clone(&first_part),
    };

    let mut part = match parse_path_part(Some(first_part), &mut tmp_path) {
        None => return Ok(path_root),
        Some(val) => val,
    };

    while let Some(val) = parse_path_part(Some(part), &mut tmp_path) {
        part = val;
    }

    Ok(path_root)
}
