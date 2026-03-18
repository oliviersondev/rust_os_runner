use core::convert::Infallible;
use crate::FrameBufferInfo;
use core::num::NonZero;
use core::ptr::{slice_from_raw_parts_mut, NonNull};
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{Dimensions, DrawTarget, Point, Size};
use embedded_graphics::primitives::Rectangle;

pub struct FrameBufferEmbeddedGraphics<'a> {
    buffer: &'a mut [u32],
    info: FrameBufferInfo,
    pixel_pitch: usize,
    bounding_box: Rectangle,
}

impl FrameBufferEmbeddedGraphics<'_> {
    /// # Safety
    /// The frame buffer must be mapped at `addr`
    pub unsafe fn new(addr: NonZero<usize>, info: FrameBufferInfo) -> Self {
        if info.bits_per_pixel as u32 == u32::BITS {
            Self {
                buffer: {
                    let mut ptr = NonNull::new(slice_from_raw_parts_mut(
                        addr.get() as *mut u32,
                        (info.pitch * info.height) as usize / size_of::<u32>(),
                    )).unwrap();
                    // Safety : this memory mapped
                    unsafe { ptr.as_mut() }
                },
                info,
                pixel_pitch: info.pitch as usize / size_of::<u32>(),
                bounding_box: Rectangle {
                    top_left: Point::zero(),
                    size: Size {
                        width: info.width.try_into().unwrap(),
                        height: info.height.try_into().unwrap(),
                    },
                },
            }
        } else {
            panic!("DrawTarget implemented for RGB888, but bpp doesn't match RGB888");
        }
    }
}

impl Dimensions for FrameBufferEmbeddedGraphics<'_> {
    fn bounding_box(&self) -> Rectangle {
        self.bounding_box
    }
}

impl DrawTarget for FrameBufferEmbeddedGraphics<'_> {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bounding_box = self.bounding_box;
        pixels
            .into_iter()
            .filter(|Pixel(point, _)| bounding_box.contains(*point))
            .for_each(|Pixel(point, color)| {
                let pixel_index = point.y as usize * self.pixel_pitch + point.x as usize;
                self.buffer[pixel_index] = self.info.pixel_info.build_pixel(&color);
            });
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box);
        let pixel = self.info.pixel_info.build_pixel(&color);
        let width = area.size.width as usize;
        let top_left_x = area.top_left.x as usize;
        for y in area.top_left.y as usize..area.top_left.y as usize + area.size.height as usize {
            let pixel_index = y * self.pixel_pitch + top_left_x;
            let pixels = &mut self.buffer[pixel_index..pixel_index + width];
            pixels.fill(pixel);
        }
        Ok(())
    }
}
