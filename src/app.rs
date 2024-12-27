use std::{io::Write, sync, thread};

use color_backtrace;
use env_logger;

pub fn init() {
    static ONCE: sync::Once = sync::Once::new();

    ONCE.call_once(|| {
        color_backtrace::install();

        env_logger::Builder::from_default_env()
            .format(|buf, record| {
                let style = buf.default_level_style(record.level());
                writeln!(
                    buf,
                    "{} {:>3} {}:{:>03} {} {style}{}{style:#} {}",
                    buf.timestamp_nanos(),
                    thread::current().id().as_u64(),
                    record.file().unwrap(),
                    record.line().unwrap(),
                    thread::current().name().unwrap_or("UNNAMED"),
                    record.level(),
                    record.args()
                )
            })
            .init();
    });
}
