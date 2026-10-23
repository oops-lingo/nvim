#![allow(unused)]
mod error;
mod logger;

use nvim_oxi::{
    self as oxi, Dictionary, Result,
    api::{notify, out_write, types::LogLevel},
};

use crate::error::IntoNvimResultExt;

#[oxi::plugin]
fn oops_lingo() -> Result<()> {
    logger::init().into_nvim()?;
    log::info!("Plugin runned!");
    Ok(())
}
