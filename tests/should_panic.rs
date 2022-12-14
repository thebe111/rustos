#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rustos::{QemuExitCode, exit_qemu, serial_println, serial_print};

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");

    assert_eq!(0, 1);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::FAILED);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[OK]");

    exit_qemu(QemuExitCode::SUCCESS);

    loop {}
}
