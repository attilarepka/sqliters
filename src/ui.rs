#![allow(dead_code)]
use anyhow::Result;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style, Stylize},
    terminal::Frame,
    text::Line,
    widgets::*,
};

use crate::model::{
    Model, ViewState, INFO_TEXT_MAIN, INFO_TEXT_TABLE, ITEM_HEIGHT, MAX_TABLE_ITEMS,
};
use crate::popup::Popup;

#[derive(Debug, Default)]
pub struct UserInterface {}

impl UserInterface {
    pub fn new() -> Result<UserInterface> {
        Ok(UserInterface {})
    }

    pub fn run(&self, frame: &mut Frame, model: &mut Model, schema: bool) {
        let rects =
            Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(frame.size());

        self.render_table(frame, model, rects[0]);

        self.render_scrollbar(frame, model, rects[0]);

        self.render_footer(frame, model, rects[1]);

        self.render_popup(frame, model, schema);
    }

    fn render_table(&self, frame: &mut Frame, model: &mut Model, area: Rect) {
        let header_style = Style::default()
            .fg(model.colors.header_fg)
            .bg(model.colors.header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(model.colors.selected_style_fg);

        let cells = model.tables[model.selected_table_id]
            .columns
            .iter()
            .map(|h| Cell::from(h.clone()));
        let header = Row::new(cells).style(header_style).height(1);

        let mut table_state = model.state.clone();
        let rows = match model.view_state {
            ViewState::Main => {
                let mut row_index = 0;
                model
                    .tables
                    .iter()
                    .flat_map(|table| {
                        table
                            .rows
                            .iter()
                            .map(|row| {
                                let color = if row_index % 2 == 0 {
                                    model.colors.normal_row_color
                                } else {
                                    model.colors.alt_row_color
                                };
                                row_index += 1;
                                let cells =
                                    row.iter().map(|value| Cell::from(value.as_str().unwrap()));
                                Row::new(cells)
                                    .style(Style::default().fg(model.colors.row_fg).bg(color))
                                    .height(ITEM_HEIGHT as u16)
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            }
            ViewState::Table => {
                let index = model.state.selected().unwrap_or(0);
                let page = index / MAX_TABLE_ITEMS;
                table_state.select(Some(index % MAX_TABLE_ITEMS));

                model.tables[model.selected_table_id]
                    .rows
                    .iter()
                    .enumerate()
                    .skip(page * MAX_TABLE_ITEMS)
                    .take(MAX_TABLE_ITEMS)
                    .map(|(row_index, row)| {
                        let color = if row_index % 2 == 0 {
                            model.colors.normal_row_color
                        } else {
                            model.colors.alt_row_color
                        };
                        let cells = row.iter().map(|cell| Cell::from(cell.as_str().unwrap()));
                        Row::new(cells)
                            .style(Style::default().fg(model.colors.row_fg).bg(color))
                            .height(ITEM_HEIGHT as u16)
                    })
                    .collect::<Vec<_>>()
            }
        };

        let constraints: Vec<_> = (0..model.tables[model.selected_table_id].columns.len())
            .map(|_| Constraint::Min(5))
            .collect();

        let t = Table::new(rows, constraints)
            .header(header)
            .highlight_style(selected_style)
            .bg(model.colors.buffer_bg)
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut table_state);
    }

    fn render_scrollbar(&self, frame: &mut Frame, model: &mut Model, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut model.scroll_state,
        );
    }

    fn render_footer(&self, frame: &mut Frame, model: &mut Model, area: Rect) {
        let info_footer = Paragraph::new(Line::from(match model.view_state {
            ViewState::Main => INFO_TEXT_MAIN,
            ViewState::Table => INFO_TEXT_TABLE,
        }))
        .style(
            Style::new()
                .fg(model.colors.row_fg)
                .bg(model.colors.buffer_bg),
        )
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(model.colors.footer_border_color)),
        );
        frame.render_widget(info_footer, area);
    }

    fn render_popup(&self, frame: &mut Frame, model: &mut Model, schema: bool) {
        if !schema {
            return;
        }
        let area = frame.size();
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };

        let popup = Popup::default()
            .content(
                model.tables[model.state.selected().unwrap_or_default()]
                    .schema
                    .as_ref()
                    .unwrap()
                    .to_string(),
            )
            .style(Style::new().yellow())
            .title(String::from("SCHEMA"))
            .title_style(Style::new().white().bold())
            .border_style(Style::new().red());
        frame.render_widget(popup, popup_area);
    }
}
