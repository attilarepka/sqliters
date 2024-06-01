mod app;
mod cli;
mod db;
mod model;
mod popup;
mod ui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = app::App::new().await?;
    app.run().await?;
    Ok(())
}
