use env_logger::fmt::{Formatter, Target};
use env_logger::{Builder, Env};
use log::Record;
use std::io::Write;

const FILTER_ENV: &'static str = "GLOBALIP_MEMO_LOG";
const WRITE_STYLE_ENV: &'static str = "GLOBALIP_MEMO_LOG_STYLE";

const DEFAULT_FILTER: &'static str = "warn";
const DEFAULT_WRITE_STYLE: &'static str = "auto";

pub fn init_logger() {
    let env = Env::default()
        .filter_or(FILTER_ENV, DEFAULT_FILTER)
        .write_style_or(WRITE_STYLE_ENV, DEFAULT_WRITE_STYLE);

    Builder::from_env(env)
        .format(|buf: &mut Formatter, record: &Record| {
            let ts = buf.timestamp();
            writeln!(
                buf,
                "{} [{}] {}:{} {} - {}",
                ts,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.target(),
                record.args(),
            )
        })
        .target(Target::Stderr)
        .init();
}
