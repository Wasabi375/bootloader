use core::{arch::asm, fmt::Write as _};

pub fn print_char(c: u8) {
    let ax = u16::from(c) | 0x0e00;
    unsafe {
        asm!("int 0x10", in("ax") ax, in("bx") 0);
    }
}

pub fn print_str(s: &str) {
    for c in s.chars() {
        if c.is_ascii() {
            print_char(c as u8);
            if c == '\n' {
                print_char(b'\r');
            }
        } else {
            print_char(b'X');
        }
    }
}

fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

pub struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print_str(s);
        Ok(())
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(Writer, "\nPANIC: {}", info);

    loop {
        hlt();
    }
}