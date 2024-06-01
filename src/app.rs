#![allow(dead_code)]
use crate::{
    cli::Args,
    db::Sqlite,
    model::{Model, Table, ViewState, ITEM_HEIGHT},
    ui::UserInterface,
};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::future::join_all;
use ratatui::{
    prelude::*,
    widgets::{ScrollbarState, TableState},
};
use serde_json::Value;
use std::{io, sync::Arc};

#[derive(Debug)]
pub struct App {
    ui: UserInterface,
    db: Arc<Sqlite>,
    model: Model,
    schema: bool,
    exit: bool,
}

impl App {
    pub async fn new() -> Result<App> {
        let args = Args::from().await;

        let db = Arc::new(Sqlite::from(&args.input_file, false).await?);

        let tables = db.tables().await?;

        let mut model = Model::default();
        let items_future: Vec<_> = tables
            .into_iter()
            .enumerate()
            .map(|(id, table)| {
                let db = db.clone();
                async move {
                    let result: Result<Table, _> = Ok::<Table, anyhow::Error>(Table {
                        name: table.clone(),
                        columns: Self::get_columns(None, &db, &ViewState::Main).await?,
                        rows: Self::get_rows(id + 1, &table, &db, &ViewState::Main).await?,
                        schema: Some(db.table_schema(table.as_str()).await?),
                    });
                    result
                }
            })
            .collect();
        let items: Vec<Result<Table, _>> = join_all(items_future).await;
        model.tables = items.into_iter().collect::<Result<Vec<Table>>>()?;
        model.scroll_state = ScrollbarState::new(model.tables.len() - 1);

        Ok(App {
            ui: UserInterface::new()?,
            db: db.clone(),
            model,
            schema: false,
            exit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            Self::reset_terminal().unwrap();
            original_hook(panic);
        }));

        let mut terminal = Self::init_terminal()?;

        while !self.exit {
            terminal.draw(|frame| self.ui.run(frame, &mut self.model, self.schema))?;
            self.handle_events().await?;
        }

        Self::reset_terminal()?;

        Ok(())
    }

    async fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).await
            }
            _ => Ok(()),
        }
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('j') | KeyCode::Down => self.model.next(),
            KeyCode::Char('k') | KeyCode::Up => self.model.previous(),
            KeyCode::Char('l') | KeyCode::Right => {
                if self.model.view_state == ViewState::Main {
                    self.schema = false;
                    self.model.selected_table_id = self.model.state.selected().unwrap_or(0);
                    self.model.state = TableState::default().with_selected(0);
                    self.model.view_state = ViewState::Table;
                    for i in 0..self.model.tables.len() {
                        self.model.tables[i].rows = Self::get_rows(
                            i + 1,
                            &self.model.tables[i].name,
                            &self.db,
                            &ViewState::Table,
                        )
                        .await?;
                        self.model.tables[i].columns = Self::get_columns(
                            Some(&self.model.tables[i].name),
                            &self.db,
                            &ViewState::Table,
                        )
                        .await?;
                    }
                    self.model.scroll_state = ScrollbarState::new(
                        (self.model.tables[self.model.selected_table_id].rows.len() - 1)
                            * ITEM_HEIGHT,
                    );
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if self.model.view_state == ViewState::Table {
                    self.model.selected_table_id = self
                        .model
                        .state
                        .selected()
                        .unwrap_or(0)
                        .min(self.model.tables.len() - 1);
                    self.model.state = TableState::default().with_selected(0);
                    self.model.view_state = ViewState::Main;
                    for i in 0..self.model.tables.len() {
                        self.model.tables[i].rows = Self::get_rows(
                            i + 1,
                            &self.model.tables[i].name,
                            &self.db,
                            &ViewState::Main,
                        )
                        .await?;
                        self.model.tables[i].columns =
                            Self::get_columns(None, &self.db, &ViewState::Main).await?;
                    }
                    self.model.scroll_state =
                        ScrollbarState::new((self.model.tables.len() - 1) * ITEM_HEIGHT);
                }
            }
            KeyCode::Char('s') | KeyCode::Char(' ') => {
                if self.model.view_state == ViewState::Main {
                    self.schema();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn schema(&mut self) {
        self.schema = !self.schema;
    }

    fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(io::stdout());

        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        Ok(terminal)
    }

    fn reset_terminal() -> Result<()> {
        disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

        Ok(())
    }

    async fn get_columns(
        name: Option<&str>,
        db: &Arc<Sqlite>,
        view: &ViewState,
    ) -> Result<Vec<String>> {
        match view {
            ViewState::Main => Ok(vec!["#", "Table", "Columns", "Rows"]
                .into_iter()
                .map(String::from)
                .collect()),
            ViewState::Table => db.table_columns(name.unwrap()).await,
        }
    }

    async fn get_rows(
        id: usize,
        table: &str,
        db: &Arc<Sqlite>,
        view: &ViewState,
    ) -> Result<Vec<Vec<Value>>> {
        match view {
            ViewState::Main => {
                let columns = db.table_columns(table).await?.join(", ");
                let rows = db.get_rows("*", table).await?;
                let len = rows.len();

                Ok(vec![vec![
                    Value::from(id.to_string()),
                    Value::from(table.to_string()),
                    Value::from(columns.clone()),
                    Value::from(len.to_string()),
                ]]
                .into_iter()
                .collect())
            }
            ViewState::Table => db.get_rows("*", table).await,
        }
    }
}
