use core::arch::asm;

/// # Safety
///
/// None of the in/out assembly wrappers are thread safe
pub unsafe fn insb(port: u16) -> u8 {
    let res: u8;
    asm!(
        "in al, dx",
        out("al") res,
        in("dx") port,
    );
    res
}

/// # Safety
///
/// None of the in/out assembly wrappers are thread safe
pub unsafe fn insw(port: u16) -> u16 {
    let mut res;
    asm!(
        "in ax, dx",
        out("ax") res,
        in("dx") port,
    );
    res
}

/// # Safety
///
/// None of the in/out assembly wrappers are thread safe
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
    );
}

/// # Safety
///
/// None of the in/out assembly wrappers are thread safe
pub unsafe fn outw(port: u16, value: u16) {
    asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") value,
    );
}

pub fn hault() -> ! {
    // SAFETY:
    // this fn is marked as unreachable, so behavior is as expected
    unsafe { asm!("hlt") };
    unreachable!()
}
