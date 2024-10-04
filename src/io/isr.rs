use core::arch::asm;

#[inline(always)]
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

#[inline(always)]
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

#[inline(always)]
pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
        );
    }
}

#[inline(always)]
pub fn outw(port: u16, value: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
        );
    }
}
