use tui::{
    buffer::Buffer, layout::Rect, style::Style, widgets::Block, widgets::Borders, widgets::Widget,
};

#[derive(Copy, Clone)]
pub struct HelpPaneState;

pub struct HelpPane {
    pub state: HelpPaneState,
    pub border_style: Style,
}

impl Widget for HelpPane {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .title("Help")
            .border_style(self.border_style)
            .borders(Borders::ALL)
            .draw(area, buf);
    }
}
