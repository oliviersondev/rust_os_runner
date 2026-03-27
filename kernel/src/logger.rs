use crate::{FrameBufferEmbeddedGraphics, WriterWithCr};
use core::fmt::{Display, Write};
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::iso_8859_16::FONT_10X20;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::Size;
use embedded_graphics::primitives::{Primitive, PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Baseline, Text};
use limine::response::FramebufferResponse;
use log::{Level, LevelFilter, Log, Record};
use owo_colors::OwoColorize;
use uart_16550::SerialPort;
use unicode_segmentation::UnicodeSegmentation;

struct Inner {
    serial_port: SerialPort,
    display: Option<DisplayData>,
}
struct KernelLogger {
    inner: spin::Mutex<Inner>,
}

struct DisplayData {
    display: FrameBufferEmbeddedGraphics<'static>,
    position: Point,
}

struct Writer<'a> {
    display: &'a mut FrameBufferEmbeddedGraphics<'static>,
    position: &'a mut Point,
    text_color: <FrameBufferEmbeddedGraphics<'a> as DrawTarget>::Color,
}

/// Represents a color in a terminal or screen. The default color may depend on if the theme is light or dark.
enum Color {
    Default,
    BrightRed,
    BrightYellow,
    BrightBlue,
    BrightCyan,
    BrightMagenta,
}

static LOGGER: KernelLogger = KernelLogger {
    inner: spin::Mutex::new(Inner {
        serial_port: unsafe { SerialPort::new(0x3F8) },
        display: None,
    }),
};

fn serial_fallback_write(args: core::fmt::Arguments<'_>) {
    unsafe {
        let mut port = SerialPort::new(0x3F8);
        port.init();
        let mut writer = WriterWithCr::new(&mut port);
        let _ = writer.write_fmt(args);
    }
}

pub fn init(frame_buffer: &'static FramebufferResponse) -> Result<(), log::SetLoggerError> {
    let mut inner = LOGGER.inner.lock();
    inner.serial_port.init();
    inner.display = frame_buffer.framebuffers().next().and_then(|fb| {
        let addr = fb.addr().addr().try_into().ok()?;
        let info = (&fb).into();
        Some(DisplayData {
            // keep unsafe scope minimal
            display: unsafe { FrameBufferEmbeddedGraphics::new(addr, info) },
            position: Point::zero(),
        })
    });
    
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

impl Inner {
    fn write_with_color(&mut self, color: Color, string: impl Display) {
        // Framebuffer path (optional)
        if let Some(display_data) = &mut self.display {
            let mut writer = Writer {
                display: &mut display_data.display,
                position: &mut display_data.position,
                text_color: match color {
                    Color::Default => Rgb888::WHITE,
                    // Mimick the ANSI escape colors
                    Color::BrightRed => Rgb888::new(255, 85, 85),
                    Color::BrightYellow => Rgb888::new(255, 255, 85),
                    Color::BrightBlue => Rgb888::new(85, 85, 255),
                    Color::BrightCyan => Rgb888::new(85, 255, 255),
                    Color::BrightMagenta => Rgb888::new(255, 85, 255),
                },
            };
            let _ = write!(writer, "{}", string);
        }
        // Serial path (always on)
        let serial_string: &dyn Display = match color {
            Color::Default => &string,
            Color::BrightRed => &string.bright_red(),
            Color::BrightYellow => &string.bright_yellow(),
            Color::BrightBlue => &string.bright_blue(),
            Color::BrightCyan => &string.bright_cyan(),
            Color::BrightMagenta => &string.bright_magenta(),
        };
        let mut writer = WriterWithCr::new(&mut self.serial_port);
        let _ = write!(writer, "{serial_string}");
    }
}

impl Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let max = log::max_level();
        max != LevelFilter::Off && metadata.level() <= max
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(mut inner) = self.inner.try_lock() {
            let level = record.level();
            inner.write_with_color(
                match level {
                    Level::Error => Color::BrightRed,
                    Level::Warn => Color::BrightYellow,
                    Level::Info => Color::BrightBlue,
                    Level::Debug => Color::BrightCyan,
                    Level::Trace => Color::BrightMagenta,
                },
                format_args!("{level:5} "),
            );
            inner.write_with_color(Color::Default, record.args());
            inner.write_with_color(Color::Default, "\r\n");
        } else {
            // Fallback if lock is busy: serial only, best effort
            serial_fallback_write(format_args!("{:5} {}\n", record.level(), record.args()));
        }

    }

    fn flush(&self) {
        // No-op: sorties série/framebuffer non bufferisées ici
    }
}

impl Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let font = FONT_10X20;
        let background_color = Rgb888::BLACK;
        for c in s.graphemes(true) {
            let height_not_seen = self.position.y + font.character_size.height as i32
                - self.display.bounding_box().size.height as i32;
            if height_not_seen > 0 {
                self.display.shift_up(height_not_seen as usize);
                self.position.y -= height_not_seen;
            }
            match c {
                "\r" => {
                    // We do not handle special cursor movements
                }
                "\n" | "\r\n" => {
                    // Fill the remaining space with background color
                    Rectangle::new(
                        *self.position,
                        Size::new(
                            self.display.bounding_box().size.width - self.position.x as u32,
                            font.character_size.height,
                        ),
                    )
                    .into_styled(
                        PrimitiveStyleBuilder::new()
                            .fill_color(background_color)
                            .build(),
                    )
                    .draw(self.display)
                    .map_err(|_| core::fmt::Error)?;
                    self.position.y += font.character_size.height as i32;
                    self.position.x = 0;
                }
                c => {
                    let style = MonoTextStyleBuilder::new()
                        .font(&font)
                        .text_color(self.text_color)
                        .background_color(background_color)
                        .build();
                    *self.position = Text::with_baseline(c, *self.position, style, Baseline::Top)
                        .draw(self.display)
                        .map_err(|_| core::fmt::Error)?;
                    if self.position.x as u32 + font.character_size.width
                        > self.display.bounding_box().size.width
                    {
                        self.position.y += font.character_size.height as i32;
                        self.position.x = 0;
                    }
                }
            }
        }
        Ok(())
    }
}
