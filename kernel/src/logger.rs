use log::{Level, Log, Record};
use core::fmt::{Display, Write};
use owo_colors::OwoColorize;
use uart_16550::SerialPort;
use crate::WriterWithCr;

struct Inner {
    serial_port: SerialPort
}
struct KernelLogger {
    inner: spin::Mutex<Inner>,
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

impl Inner {
    fn write_with_color(
        &mut self,
        color: Color,
        string: impl Display,
    ) {
        let string: &dyn Display = match color {
            Color::Default => &string,
            Color::BrightRed => &string.bright_red(),
            Color::BrightYellow => &string.bright_yellow(),
            Color::BrightBlue => &string.bright_blue(),
            Color::BrightCyan => &string.bright_cyan(),
            Color::BrightMagenta => &string.bright_magenta(),
        };
        let mut writer = WriterWithCr::new(&mut self.serial_port);
        write!(writer, "{string}").unwrap();
    }
}

impl Log for KernelLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        todo!()
    }

    fn log(&self, record: &Record) {
        let mut inner = self.inner.try_lock().unwrap();
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
    }

    fn flush(&self) {
        todo!()
    }
}
