use crate::fs::file::fclose;
use crate::fs::file::fopen;
use crate::fs::file::fread;
use crate::fs::file::fstat;
use crate::println;
use crate::status::ErrorCode;
use alloc::format;
use alloc::string::String;

macro_rules! log {
    ($($arg:tt)*) => {
        println!("[fat16_test] {}", format!($($arg)*));
    };
}

pub fn fat16_test() -> Result<(), ErrorCode> {
    log!("Attempting to open 1:/HELLO.TXT...");
    let fd = fopen("1:/HELLO.TXT", "r")?;

    log!("Attempting to read 1:/HELLO.TXT...");
    let mut buf = [0; 8];
    fread(&mut buf, 8, 1, fd)?;
    let result = String::from_utf8(buf.to_vec()).unwrap();
    assert!(result == "Welcome\n");

    log!("Attempting to stat 1:/HELLO.TXT...");
    let stats = fstat(fd)?;
    assert!(stats.filesize == 8);
    let _ = fclose(fd);

    log!("Successfully tested fat16");
    Ok(())
}
