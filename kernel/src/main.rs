#![no_std]
#![no_main]

use core::panic::PanicInfo;
use limine::BaseRevision;
use limine::request::{RequestsEndMarker, RequestsStartMarker};
use kernel::hlt_loop;
use uart_16550::SerialPort;
use core::fmt::Write;

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[unsafe(link_section = ".limine_requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// Define the stand and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".limine_requests_start")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".limine_requests_end")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn entry_point_from_limine() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    writeln!(serial_port, "Hello World!\r").unwrap();

    hlt_loop();
}
// TODO option de build pour pouvoir booter avec bootloader minimalist https://github.com/rust-osdev/bootloader
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt_loop();
}
