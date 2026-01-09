#![allow(dead_code)]
use crate::model::{Model, ViewState, ITEM_HEIGHT, MAX_TABLE_ITEMS};
use crate::popup::Popup;
use ratatui::text::Text;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        Table,
    },
    Frame,
};

#[derive(Debug, Default)]
pub struct UserInterface {}

impl UserInterface {
    pub fn new() -> UserInterface {
        UserInterface {}
    }

    pub fn run(&self, frame: &mut Frame, model: &Model) {
        let _ = self;

        let schema = model.is_schema_enabled();
        let rects =
            Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(frame.area());

        Self::render_table(frame, model, rects[0]);

        Self::render_scrollbar(frame, model, rects[0]);

        Self::render_footer(frame, model, rects[1]);

        Self::render_popup(frame, model, schema);
    }

    fn render_table(frame: &mut Frame, model: &Model, area: Rect) {
        let header_style = Style::default().bg(model.colors().header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(model.colors().selected_style_fg);
        let highlight_column_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(model.colors().highlight_column_fg);

        let cells = model
            .get_table_columns()
            .iter()
            .enumerate()
            .map(|(i, header)| {
                let header_style = if model.is_column_enabled() && i == model.active_column() {
                    Style::default().fg(model.colors().header_fg)
                } else {
                    Style::default().fg(model.colors().selected_style_fg)
                };

                Cell::from(Text::from(header.clone()).centered()).style(header_style)
            });
        let header = Row::new(cells).style(header_style).height(1);

        let mut table_state = *model.state();
        let index = model.state().selected().unwrap_or(0);
        table_state.select(Some(index % MAX_TABLE_ITEMS));
        let rows = match model.view_state() {
            ViewState::Main => Self::render_main_state(model, highlight_column_style),
            ViewState::Table => Self::render_table_state(model, highlight_column_style),
        };

        let constraints: Vec<_> = (0..model.get_table_columns().len())
            .map(|column| {
                if model.is_column_enabled() && column == model.active_column() {
                    Constraint::Min(model.longest_in_column())
                } else {
                    Constraint::Min(5)
                }
            })
            .collect();
        let bar = " █ ";
        let t = Table::new(rows, constraints)
            .header(header)
            .row_highlight_style(selected_style)
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                bar.into(),
                "".into(),
            ]))
            .bg(model.colors().buffer_bg)
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut table_state);
    }

    fn render_main_state(model: &Model, highlight_column_style: Style) -> Vec<Row<'_>> {
        let mut row_index = 0;
        model
            .tables()
            .iter()
            .flat_map(|table| {
                table
                    .rows()
                    .iter()
                    .map(|row| {
                        let color = if row_index % 2 == 0 {
                            model.colors().normal_row_color
                        } else {
                            model.colors().alt_row_color
                        };
                        row_index += 1;
                        let cells = row.iter().enumerate().map(|(i, cell)| {
                            let cell_style =
                                if model.is_column_enabled() && i == model.active_column() {
                                    highlight_column_style
                                } else {
                                    Style::default().fg(model.colors().row_fg).bg(color)
                                };
                            Cell::from(
                                Text::from(format!("\n{}\n", cell.as_str().unwrap())).centered(),
                            )
                            .style(cell_style)
                        });
                        Row::new(cells)
                            .style(Style::default().fg(model.colors().row_fg).bg(color))
                            .height(ITEM_HEIGHT)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn render_table_state(model: &Model, highlight_column_style: Style) -> Vec<Row<'_>> {
        let index = model.state().selected().unwrap_or(0);
        let page = index / MAX_TABLE_ITEMS;
        model
            .get_table_rows()
            .iter()
            .enumerate()
            .skip(page * MAX_TABLE_ITEMS)
            .take(MAX_TABLE_ITEMS)
            .map(|(row_index, row)| {
                let color = if row_index % 2 == 0 {
                    model.colors().normal_row_color
                } else {
                    model.colors().alt_row_color
                };
                let cells = row.iter().enumerate().map(|(i, cell)| {
                    let cell_style = if model.is_column_enabled() && i == model.active_column() {
                        highlight_column_style
                    } else {
                        Style::default().fg(model.colors().row_fg).bg(color)
                    };
                    Cell::from(Text::from(format!("\n{}\n", cell.as_str().unwrap())).centered())
                        .style(cell_style)
                });
                Row::new(cells)
                    .style(Style::default().fg(model.colors().row_fg).bg(color))
                    .height(ITEM_HEIGHT)
            })
            .collect::<Vec<_>>()
    }

    fn render_scrollbar(frame: &mut Frame, model: &Model, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut model.scroll_state().clone(),
        );
    }

    fn render_footer(frame: &mut Frame, model: &Model, area: Rect) {
        let info_footer = Paragraph::new(model.get_info_text())
            .style(
                Style::new()
                    .fg(model.colors().row_fg)
                    .bg(model.colors().buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(model.colors().footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }

    fn render_popup(frame: &mut Frame, model: &Model, schema: bool) {
        if !schema {
            return;
        }
        let area = frame.area();
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };

        let popup = Popup::default()
            .content((*model.table_schema().as_ref().unwrap()).to_string())
            .style(Style::new().yellow())
            .title(String::from("SCHEMA"))
            .title_style(Style::new().white().bold())
            .border_style(Style::new().red());
        frame.render_widget(popup, popup_area);
    }
}
