#![allow(dead_code)]
use crate::db::Sqlite;
use anyhow::Result;
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;
use style::palette::tailwind;
use style::Color;

pub const ITEM_HEIGHT: usize = 4;
pub const MAX_TABLE_ITEMS: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub struct TableColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub selected_header_fg: Color,
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub footer_border_color: Color,
    pub highlight_column_fg: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            selected_header_fg: tailwind::SLATE.c800,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
            highlight_column_fg: color.c800,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    name: String,
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
    schema: String,
}

impl Table {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn columns(&self) -> &Vec<String> {
        &self.columns
    }

    pub fn rows(&self) -> &Vec<Vec<Value>> {
        &self.rows
    }
    pub fn schema(&self) -> &str {
        &self.schema.as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewState {
    Main,
    Table,
}

#[derive(Debug, Clone)]
pub struct Model {
    tables: Vec<Table>,
    selected_table_id: usize,
    state: TableState,
    scroll_state: ScrollbarState,
    active_column: usize,
    colors: TableColors,
    view_state: ViewState,
    schema: bool,
    column: bool,
    db: Sqlite,
}

impl Model {
    pub fn new(db: Sqlite) -> Result<Model> {
        Ok(Model {
            tables: Vec::new(),
            selected_table_id: 0,
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::default(),
            active_column: 0,
            colors: TableColors::new(&tailwind::TEAL),
            view_state: ViewState::Main,
            schema: false,
            column: false,
            db,
        })
    }
    pub async fn initialize(&mut self) -> Result<()> {
        let tables = self.db.tables().await?;
        let items_future: Vec<_> = tables
            .into_iter()
            .enumerate()
            .map(|(id, table)| {
                let db = self.db.clone();
                async move {
                    let result: Result<Table, _> = Ok::<Table, anyhow::Error>(Table {
                        name: table.clone(),
                        columns: Self::columns(None, &db, &ViewState::Main).await?,
                        rows: Self::rows(id + 1, &table, &db, &ViewState::Main).await?,
                        schema: db.table_schema(table.as_str()).await?,
                    });
                    result
                }
            })
            .collect();
        let items: Vec<Result<Table, _>> = futures::future::join_all(items_future).await;
        self.tables = items.into_iter().collect::<Result<Vec<Table>>>()?;
        self.scroll_state =
            ScrollbarState::new(self.tables.len().checked_sub(1).unwrap_or_default());

        Ok(())
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => match self.view_state {
                ViewState::Main => {
                    if i >= self.tables.len().checked_sub(1).unwrap_or(0) {
                        0
                    } else {
                        i + 1
                    }
                }
                ViewState::Table => match self.tables.get(self.selected_table_id) {
                    Some(table) => {
                        if i >= table.rows().len().checked_sub(1).unwrap_or(0) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                },
            },
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => match self.view_state {
                ViewState::Main => {
                    if i == 0 {
                        self.tables.len().checked_sub(1).unwrap_or(0)
                    } else {
                        i - 1
                    }
                }
                ViewState::Table => match self.tables.get(self.selected_table_id) {
                    Some(table) => {
                        if i == 0 {
                            table.rows().len().checked_sub(1).unwrap_or(0)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                },
            },
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub async fn switch_to_table_view(&mut self) -> Result<()> {
        if self.view_state == ViewState::Main {
            self.schema = false;
            self.column = false;
            self.active_column = 0;
            self.selected_table_id = self.state.selected().unwrap_or(0);
            self.state = TableState::default().with_selected(0);
            self.view_state = ViewState::Table;

            for i in 0..self.tables.len() {
                if let Some(table) = self.tables.get_mut(i) {
                    table.rows =
                        Self::rows(i + 1, &table.name, &self.db, &ViewState::Table).await?;
                    table.columns =
                        Self::columns(Some(&table.name), &self.db, &ViewState::Table).await?;
                }
            }

            if let Some(selected_table) = self.tables.get(self.selected_table_id) {
                self.scroll_state =
                    ScrollbarState::new((selected_table.rows().len() - 1) * ITEM_HEIGHT);
            }
        }
        Ok(())
    }

    pub async fn switch_to_main_view(&mut self) -> Result<()> {
        if self.view_state == ViewState::Table {
            self.column = false;
            self.active_column = 0;
            self.selected_table_id = self
                .state
                .selected()
                .unwrap_or(0)
                .min(self.tables.len().checked_sub(1).unwrap_or(0));
            self.state = TableState::default().with_selected(0);
            self.view_state = ViewState::Main;
            for i in 0..self.tables.len() {
                self.tables[i].rows =
                    Self::rows(i + 1, &self.tables[i].name, &self.db, &ViewState::Main).await?;
                self.tables[i].columns = Self::columns(None, &self.db, &ViewState::Main).await?;
            }
            self.scroll_state =
                ScrollbarState::new((self.tables.len().checked_sub(1).unwrap_or(0)) * ITEM_HEIGHT);
        }
        Ok(())
    }

    pub fn tables(&self) -> &[Table] {
        &self.tables
    }

    pub fn table_schema(&self) -> Option<&str> {
        self.tables
            .get(self.selected_table_id)
            .map(|table| table.schema())
    }

    pub fn get_table_columns(&self) -> &[String] {
        self.tables
            .get(self.selected_table_id)
            .map(|table| table.columns.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_table_rows(&self) -> Vec<&[Value]> {
        self.tables
            .get(self.selected_table_id)
            .map(|table| {
                table
                    .rows()
                    .iter()
                    .map(|row| row.as_slice())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![&[]])
    }

    pub fn longest_in_column(&self) -> u16 {
        const WIDTH_PERCENTAGE: f32 = 1.1;

        if let Some(table) = self.tables.get(self.selected_table_id) {
            if let Some(active_column_index) = table.columns.get(self.active_column) {
                let longest_row = table
                    .rows()
                    .iter()
                    .filter_map(|row| row.get(self.active_column))
                    .filter_map(|value| value.as_str().map(|s| s.len()))
                    .max()
                    .unwrap_or(0);

                let longest_column = active_column_index.as_str().len();

                return (longest_row.max(longest_column) as f32 * WIDTH_PERCENTAGE) as u16;
            }
        }

        0
    }

    pub fn view_state(&self) -> ViewState {
        self.view_state.clone()
    }

    pub fn colors(&self) -> &TableColors {
        &self.colors
    }

    pub fn selected_table_id(&self) -> usize {
        self.selected_table_id
    }

    pub fn state(&self) -> &TableState {
        &self.state
    }

    pub fn scroll_state(&self) -> &ScrollbarState {
        &self.scroll_state
    }

    pub fn is_schema_enabled(&self) -> bool {
        self.schema
    }

    pub fn toggle_schema(&mut self) {
        if self.view_state == ViewState::Main {
            self.schema = !self.schema;
        }
    }

    pub fn is_column_enabled(&self) -> bool {
        self.column
    }

    pub fn toggle_column(&mut self) {
        self.column = !self.column;
    }

    pub fn active_column(&self) -> usize {
        self.active_column
    }

    pub fn next_column(&mut self) {
        if self.is_column_enabled() {
            self.active_column = (self.active_column + 1)
                % self
                    .tables
                    .get(self.selected_table_id)
                    .map_or(0, |table| table.columns.len());
        }
    }

    pub fn previous_column(&mut self) {
        if self.is_column_enabled() {
            self.active_column = if self.active_column == 0 {
                self.tables
                    .get(self.selected_table_id)
                    .map_or(0, |table| table.columns.len())
                    - 1
            } else {
                self.active_column - 1
            };
        }
    }

    pub fn get_info_text(&self) -> String {
        let mut result =
            String::from("(Esc) quit | (↑) move up | (↓) move down | (⇧ S) toggle column select");
        match self.view_state {
            ViewState::Main => {
                result.push_str(" | (Space) toggle schema (→) table view");
            }
            ViewState::Table => {
                result.push_str(" | (←) main view");
            }
        }

        if self.is_column_enabled() {
            result.push_str(" | (⇧ ←) previous column | (⇧ →) next column");
        }

        result
    }

    async fn columns(name: Option<&str>, db: &Sqlite, view: &ViewState) -> Result<Vec<String>> {
        match view {
            ViewState::Main => Ok(vec!["#", "Table", "Columns", "Rows"]
                .into_iter()
                .map(String::from)
                .collect()),
            ViewState::Table => db.table_columns(name.unwrap()).await,
        }
    }

    async fn rows(
        id: usize,
        table: &str,
        db: &Sqlite,
        view: &ViewState,
    ) -> Result<Vec<Vec<Value>>> {
        match view {
            ViewState::Main => {
                let columns = db.table_columns(table).await?;
                let rows = db.get_rows("*", table).await?;
                let len = rows.len();

                Ok(vec![vec![
                    Value::from(id.to_string()),
                    Value::from(table.to_string()),
                    Value::from(columns.len().to_string()),
                    Value::from(len.to_string()),
                ]]
                .into_iter()
                .collect())
            }
            ViewState::Table => db.get_rows("*", table).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn initialize_main_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        assert!(!model.is_schema_enabled());
        assert!(!model.is_column_enabled());
        assert_eq!(model.tables().len(), 0);
        assert_eq!(model.view_state(), ViewState::Main);
        assert_eq!(model.selected_table_id(), 0);
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
        assert_eq!(model.colors(), &TableColors::new(&tailwind::TEAL));
        assert_eq!(model.longest_in_column(), 0);
        assert_eq!(model.active_column(), 0);
    }

    #[tokio::test]
    async fn initialize_table_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        model.switch_to_table_view().await.unwrap();
        assert_eq!(model.view_state(), ViewState::Table);
        assert_eq!(model.selected_table_id(), 0);
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
        assert_eq!(model.colors(), &TableColors::new(&tailwind::TEAL));
        assert_eq!(model.longest_in_column(), 0);
        assert!(!model.is_schema_enabled());
        assert!(!model.is_column_enabled());
    }

    #[tokio::test]
    async fn main_view_empty_next() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.next();
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
    }

    #[tokio::test]
    async fn table_view_empty_next() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.switch_to_table_view().await.unwrap();
        model.next();
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
    }

    #[tokio::test]
    async fn main_view_empty_previous() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.previous();
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
    }

    #[tokio::test]
    async fn table_view_empty_previous() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.switch_to_table_view().await.unwrap();
        model.previous();
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
    }

    #[tokio::test]
    async fn switch_to_main_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.switch_to_table_view().await.unwrap();
        assert_eq!(model.view_state(), ViewState::Table);
        assert_eq!(model.selected_table_id(), 0);
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
        assert_eq!(model.colors(), &TableColors::new(&tailwind::TEAL));
        assert_eq!(model.longest_in_column(), 0);
        model.switch_to_main_view().await.unwrap();
        assert_eq!(model.view_state(), ViewState::Main);
        assert_eq!(model.selected_table_id(), 0);
        assert_eq!(model.state().selected(), Some(0));
        assert_eq!(model.scroll_state(), &ScrollbarState::default());
        assert_eq!(model.colors(), &TableColors::new(&tailwind::TEAL));
        assert_eq!(model.longest_in_column(), 0);
    }

    #[tokio::test]
    async fn toggle_schema_main_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        assert!(!model.is_schema_enabled());
        model.toggle_schema();
        assert!(model.is_schema_enabled());
    }

    #[tokio::test]
    async fn toggle_schema_table_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.switch_to_table_view().await.unwrap();
        assert!(!model.is_schema_enabled());
        model.toggle_schema();
        assert!(!model.is_schema_enabled());
    }

    #[tokio::test]
    async fn toggle_column_main_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        assert!(!model.is_column_enabled());
        model.toggle_column();
        assert!(model.is_column_enabled());
    }

    #[tokio::test]
    async fn toggle_column_table_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        assert!(!model.is_column_enabled());
        model.toggle_column();
        assert!(model.is_column_enabled());
    }

    #[tokio::test]
    async fn info_text_main_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        assert_eq!(
            model.get_info_text(),
            "(Esc) quit | (↑) move up | (↓) move down | (⇧ S) toggle column select | (Space) toggle schema (→) table view"
        );
    }
    #[tokio::test]
    async fn info_text_with_column_main_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.toggle_column();
        assert_eq!(
            model.get_info_text(),
            "(Esc) quit | (↑) move up | (↓) move down | (⇧ S) toggle column select | (Space) toggle schema (→) table view | (⇧ ←) previous column | (⇧ →) next column"
        );
    }

    #[tokio::test]
    async fn info_text_table_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.switch_to_table_view().await.unwrap();
        assert_eq!(
            model.get_info_text(),
            "(Esc) quit | (↑) move up | (↓) move down | (⇧ S) toggle column select | (←) main view"
        );
    }

    #[tokio::test]
    async fn info_text_column_table_view() {
        let db = Sqlite::new().await.unwrap();
        let mut model = Model::new(db).unwrap();
        assert!(model.initialize().await.is_ok());
        model.switch_to_table_view().await.unwrap();
        model.toggle_column();
        assert_eq!(
            model.get_info_text(),
            "(Esc) quit | (↑) move up | (↓) move down | (⇧ S) toggle column select | (←) main view | (⇧ ←) previous column | (⇧ →) next column"
        );
    }
}
