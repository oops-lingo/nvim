use anyhow::{Context, Result};
use chrono::Local;
use log::{LevelFilter, Log, set_boxed_logger, set_max_level};
use nvim_oxi::{
    api::{
        Buffer, command, create_buf, get_current_win, opts::OptionOpts, set_current_win,
        set_option_value,
    },
    conversion::ToObject,
    libuv::AsyncHandle,
    schedule,
};
use std::{
    fmt::Display,
    sync::mpsc::{Sender, channel},
};

use crate::error::IntoNvimResultExt;

#[derive(Clone)]
struct Buf {
    buffer: Buffer,
    opts: OptionOpts,
}

impl Buf {
    fn new() -> Result<Self> {
        let buffer = create_buf(false, true)?;
        let opts = OptionOpts::builder().buffer(buffer.clone()).build();
        let buf = Self { buffer, opts };

        buf.set_option("bufhidden", "hide")?;
        buf.set_option("buftype", "nofile")?;
        buf.set_option("filetype", "log")?;
        buf.set_option("modifiable", false)?;
        buf.set_option("swapfile", false)?;
        Self::init_win(&buf.buffer).context("Initialize window")?;
        Ok(buf)
    }

    fn set_option(&self, name: &str, value: impl ToObject + Display + Copy) -> Result<()> {
        set_option_value(name, value, &self.opts)
            .with_context(|| format!("Set buffer option {name} with value {value}"))?;
        Ok(())
    }

    fn init_win(buffer: &Buffer) -> Result<()> {
        let original = get_current_win();
        let width = (original.get_width()? / 3).max(20);

        command("botright vsplit")?;

        let mut window = get_current_win();
        window.set_buf(buffer)?;
        window.set_width(width)?;

        let win_opts = OptionOpts::builder().win(window.clone()).build();

        set_option_value("winfixwidth", true, &win_opts)?;

        set_current_win(&original)?;
        Ok(())
    }

    fn write(&mut self, message: &str) -> Result<()> {
        let end = self.buffer.line_count()?;
        self.set_option("modifiable", true)?;
        self.buffer
            .set_lines(end..end, false, [message])
            .with_context(|| format!("Set line into buffer by position {end}"))?;
        self.set_option("modifiable", false)?;
        Ok(())
    }
}

struct Logger {
    sender: Sender<String>,
    handle: AsyncHandle,
}

impl Logger {
    fn send_log(&self, message: String) -> Result<()> {
        self.sender.send(message)?;
        self.handle.send()?;
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let time = Local::now().format("%H:%M:%S%.3f");
        let level = record.level().to_string().to_uppercase();

        if let Err(err) = self.send_log(format!("{time} [{level}] {}", record.args())) {
            eprintln!("Error: {err:?}");
        }
    }

    fn flush(&self) {}
}

fn init_logger(buf: Buf) -> Result<()> {
    let (tx, rx) = channel::<String>();
    let handle = AsyncHandle::new(move || {
        while let Ok(message) = rx.try_recv() {
            let mut buf = buf.clone();
            schedule(move |_| buf.write(&message).into_nvim());
        }
    })
    .context("Create async handle")?;
    let logger = Logger { sender: tx, handle };
    set_boxed_logger(Box::new(logger)).context("Set global logger")?;
    set_max_level(LevelFilter::Trace);
    Ok(())
}

pub fn init() -> Result<()> {
    let mut buf = Buf::new().context("Initialize logger buffer")?;
    if let Err(err) = init_logger(buf.clone()) {
        buf.write(&format!("[ERROR] Init logger: {err:#}"));
    }

    Ok(())
}
