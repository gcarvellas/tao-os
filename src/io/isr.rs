use core::arch::asm;

pub fn insb(port: u16) -> u8 {
    let res: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") res,
            in("dx") port,
        );
    }
    res
}

pub fn insw(port: u16) -> u16 {
    let mut res;
    unsafe {
        asm!(
            "in ax, dx",
            out("ax") res,
            in("dx") port,
        );
    }
    res
}

pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
        );
    }
}

pub fn outw(port: u16, value: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
        );
    }
}
