use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::text::Text;
use ratatui::widgets::Block;
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
    let block = Block::bordered();
    let area = _center(
        frame.size(),
        Constraint::Length(text.width() as u16 + 2),
        Constraint::Length(3),
    );
    frame.render_widget(block.clone(), area);
    frame.render_widget(text, block.inner(area));
}
// (block "title" "content"
//   (style
//     (title_align center)
//     (title_color (#00ffff #000000))
//     (borders all)))

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
        _render(&mut ui.terminal.get_frame());

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
