mod app;
mod cli;
mod database;
mod model;
mod popup;
mod ui;

use anyhow::Result;

use crate::database::SqliteDb;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::from();
    let db = SqliteDb::connect(&args.input, false).await?;

    let mut app = app::App::new(db).await?;
    app.run().await?;

    Ok(())
}
