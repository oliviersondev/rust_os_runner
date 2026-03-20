#![no_std]

mod frame_buffer_info;
mod rgb_pixel_info;
mod frame_buffer_embedded_graphics;
mod limine_requests;
mod logger;
mod writer_with_cr;

pub use rgb_pixel_info::RgbPixelInfo;
pub use frame_buffer_info::FrameBufferInfo;
pub use limine_requests::FRAME_BUFFER_REQUEST;
pub use frame_buffer_embedded_graphics::FrameBufferEmbeddedGraphics;
pub use writer_with_cr::WriterWithCr;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
