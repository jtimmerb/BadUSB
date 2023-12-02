use anyhow::{Result};

mod app;
mod usb;
mod detector;
mod helper;

fn main() -> Result<()> {
    let app = app::App::new()?;
    app.run()?;
    Ok(())
}