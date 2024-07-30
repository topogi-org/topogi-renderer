use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use topogi_renderer::Result;
use topogi_renderer::UIEngine;

fn main() -> Result<()> {
    let mut ui = UIEngine::new().unwrap();
    let source = r#"
        (block "hello world"
            (stack vertical
                ((length 20) (block "1" "hello"))
                ((length 20) (block "2" "hello"))
                ((length 20) 
                (block "3"
                    (stack horizontal
                        ((length 20) (block "3-1" "hello"))
                        ((length 20) (block "3-2" "hello"))
                    )
                ))
            )
        )
    "#;
    let mut parser = topogi_lang::parser::Parser::new(source);
    let exp = parser.parse_exp().unwrap();

    loop {
        ui.render(&exp).unwrap();

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
