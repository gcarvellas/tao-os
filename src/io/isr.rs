use core::arch::asm;

#[inline(always)]
pub fn insb(port: u16) -> u8 {
    let value: u8 = 0;
    unsafe {
        asm!(
            "in al, dx",
            inout("al") value => _,
            in("dx") port,
        );
    }
    value
}

#[inline(always)]
pub fn insw(port: u16) -> u16 {
    let value: u16 = 0;
    unsafe {
        asm!(
            "in ax, dx",
            inout("ax") value => _,
            in("dx") port,
        );
    }
    value
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
