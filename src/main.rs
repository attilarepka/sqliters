mod app;
mod cli;
mod db;
mod model;
mod popup;
mod ui;

use anyhow::Result;
use db::Sqlite;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::from();
    let db = Sqlite::from(&args.input_file, false).await?;

    let mut app = app::App::new(db).await?;
    app.run().await?;

    Ok(())
}
