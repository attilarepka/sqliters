#![allow(dead_code)]
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;
use style::palette::tailwind;
use style::Color;

pub const ITEM_HEIGHT: usize = 4;

pub const INFO_TEXT_MAIN: &str =
    "(Esc) quit | (Space) toggle schema | (↑) move up | (↓) move down | (→) table view";
pub const INFO_TEXT_TABLE: &str = "(Esc) quit | (↑) move up | (↓) move down | (←) main view";
pub const MAX_TABLE_ITEMS: usize = 100;

#[derive(Debug, Clone)]
pub struct TableColors {
    pub buffer_bg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub selected_style_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub schema: Option<String>,
}

impl Table {
    pub const fn schema(&self) -> &Option<String> {
        &self.schema
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewState {
    Main,
    Table,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub tables: Vec<Table>,
    pub selected_table_id: usize,
    pub state: TableState,
    pub scroll_state: ScrollbarState,
    pub colors: TableColors,
    pub view_state: ViewState,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            tables: Vec::new(),
            selected_table_id: 0,
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::default(),
            colors: TableColors::new(&tailwind::TEAL),
            view_state: ViewState::Main,
        }
    }
}

impl Model {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => match self.view_state {
                ViewState::Main => {
                    if i >= self.tables.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                ViewState::Table => {
                    if i >= self.tables[self.selected_table_id].rows.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
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
                        self.tables.len() - 1
                    } else {
                        i - 1
                    }
                }
                ViewState::Table => {
                    if i == 0 {
                        self.tables[self.selected_table_id].rows.len() - 1
                    } else {
                        i - 1
                    }
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }
}
