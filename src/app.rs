#![allow(dead_code)]
use crate::{db::Sqlite, model::Model, ui::UserInterface};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

#[derive(Debug)]
pub struct App {
    ui: UserInterface,
    model: Model,
    exit: bool,
}

impl App {
    pub async fn new(db: Sqlite) -> Result<App> {
        let mut model = Model::new(db)?;
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
            terminal.draw(|frame| self.ui.run(frame, &self.model))?;
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

#[cfg(test)]
mod tests {
    use crate::model::ViewState;

    use super::*;

    async fn create_test_db() -> Sqlite {
        let db = Sqlite::new().await.unwrap();
        db.create_table("test", format!("{} INTEGER", "id").as_str())
            .await
            .unwrap();
        db.insert_rows("test", "id", &vec!["1", "2", "3"])
            .await
            .unwrap();
        db.create_table("test2", format!("{} INTEGER", "id").as_str())
            .await
            .unwrap();
        db.insert_rows("test2", "id", &vec!["1", "2", "3"])
            .await
            .unwrap();
        db
    }

    #[tokio::test]
    async fn handle_key_events() {
        let db = create_test_db().await;
        let mut app = App::new(db).await.unwrap();

        app.handle_key_event(KeyEvent::new(KeyCode::Char('q'), event::KeyModifiers::NONE))
            .await
            .unwrap();
        assert!(app.exit);

        app.handle_key_event(KeyEvent::new(KeyCode::Char('j'), event::KeyModifiers::NONE))
            .await
            .unwrap();
        assert_eq!(app.model.state().selected().unwrap(), 1);

        app.handle_key_event(KeyEvent::new(KeyCode::Char('k'), event::KeyModifiers::NONE))
            .await
            .unwrap();
        assert_eq!(app.model.state().selected().unwrap(), 0);

        app.handle_key_event(KeyEvent::new(KeyCode::Char('l'), event::KeyModifiers::NONE))
            .await
            .unwrap();
        assert_eq!(app.model.view_state(), ViewState::Table);

        app.handle_key_event(KeyEvent::new(KeyCode::Char('h'), event::KeyModifiers::NONE))
            .await
            .unwrap();
        assert_eq!(app.model.view_state(), ViewState::Main);

        app.handle_key_event(KeyEvent::new(KeyCode::Char('s'), event::KeyModifiers::NONE))
            .await
            .unwrap();
        assert!(app.model.is_schema_enabled());

        app.handle_key_event(KeyEvent::new(
            KeyCode::Char('S'),
            event::KeyModifiers::SHIFT,
        ))
        .await
        .unwrap();
        assert!(app.model.is_column_enabled());

        app.handle_key_event(KeyEvent::new(
            KeyCode::Char('L'),
            event::KeyModifiers::SHIFT,
        ))
        .await
        .unwrap();
        assert_eq!(app.model.active_column(), 1);

        app.handle_key_event(KeyEvent::new(
            KeyCode::Char('H'),
            event::KeyModifiers::SHIFT,
        ))
        .await
        .unwrap();
        assert_eq!(app.model.active_column(), 0);
    }
}
