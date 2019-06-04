use tui::{buffer::Buffer, layout::Rect, widgets::Block, widgets::Borders, widgets::Widget};

#[derive(Copy, Clone)]
pub struct HelpPaneState;

pub struct HelpPane {
    pub state: HelpPaneState,
    pub borders: Borders,
}

impl Widget for HelpPane {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .title("Help")
            .borders(self.borders)
            .draw(area, buf);
    }
}
