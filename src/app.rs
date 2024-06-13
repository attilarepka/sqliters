#![allow(dead_code)]
use crate::{cli::Args, db::Sqlite, model::Model, ui::UserInterface};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{io, sync::Arc};

#[derive(Debug)]
pub struct App {
    ui: UserInterface,
    model: Model,
    exit: bool,
}

impl App {
    pub async fn new() -> Result<App> {
        let args = Args::from().await;

        let db = Arc::new(Sqlite::from(&args.input_file, false).await?);

        let mut model = Model::new(db.clone())?;
        model.initialize().await?;

        Ok(App {
            ui: UserInterface::new()?,
            model,
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
            terminal.draw(|frame| self.ui.run(frame, &mut self.model))?;
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
        match key_event {
            KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Esc,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.exit(),
            KeyEvent {
                code: KeyCode::Char('j') | KeyCode::Down,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.model.next(),
            KeyEvent {
                code: KeyCode::Char('k') | KeyCode::Up,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.model.previous(),
            KeyEvent {
                code: KeyCode::Char('l') | KeyCode::Right,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.model.switch_to_table_view().await?,
            KeyEvent {
                code: KeyCode::Char('h') | KeyCode::Left,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.model.switch_to_main_view().await?,
            KeyEvent {
                code: KeyCode::Char('s') | KeyCode::Char(' '),
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.model.toggle_schema(),
            KeyEvent {
                code: KeyCode::Char('H') | KeyCode::Left,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => self.model.previous_column(),
            KeyEvent {
                code: KeyCode::Char('L') | KeyCode::Right,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => self.model.next_column(),
            KeyEvent {
                code: KeyCode::Char('S'),
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => self.model.toggle_column(),
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
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
}
