use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Debug, Default)]
pub struct Popup {
    title: String,
    content: String,
    border_style: Style,
    title_style: Style,
    style: Style,
}

impl Popup {
    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
    pub fn content(mut self, content: String) -> Self {
        self.content = content;
        self
    }
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Popup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::new()
            .title(Line::from(self.title))
            .title_alignment(Alignment::Center)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}
