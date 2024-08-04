use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::text::Text;
use ratatui::{layout::*, Frame};
use topogi_renderer::Result;
use topogi_renderer::UIEngine;

fn _center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn _render(frame: &mut Frame) {
    let text = Text::raw("Hello world!");
    let area = _center(
        frame.size(),
        Constraint::Length(text.width() as u16),
        Constraint::Length(1),
    );
    frame.render_widget(text, area);
}

fn main() -> Result<()> {
    let mut ui = UIEngine::new().unwrap();
    let source = r#"
        (layer
            (block "title1" "content1")
            (block "title2" "content2"))
    "#;
    let mut parser = topogi_lang::parser::Parser::new(source);
    let exp = parser.parse_exp().unwrap();

    loop {
        ui.render_layer(&exp).unwrap();

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    ui.shutdown()
}
