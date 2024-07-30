use std::io::{stdout, Stdout};

pub mod render_tree;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Layout, Rect},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};
use render_tree::{create_render_tree, RenderTree};
use topogi_lang::ast::Exp;

#[derive(Debug)]
pub struct UIEngine {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Debug)]
pub enum RenderError {
    InvalidBlock(Exp),
    InvalidStack(Exp),
    InvalidExp(Exp),
    NotConstraint(Exp),
    InvalidDirection(String),
    IO(std::io::Error),
}

impl From<std::io::Error> for RenderError {
    fn from(err: std::io::Error) -> Self {
        RenderError::IO(err)
    }
}

pub type Result<T> = std::result::Result<T, RenderError>;

impl UIEngine {
    pub fn new() -> Result<Self> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear()?;
        Ok(UIEngine { terminal })
    }

    pub fn render(&mut self, exp: &Exp) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.size();
            match create_render_tree(exp) {
                Ok(render_tree) => render(&render_tree, frame, area),
                Err(err) => {
                    let error = format!("Error: {:?}", err);
                    frame.render_widget(Paragraph::new(error).block(Block::bordered()), area)
                }
            }
        })?;

        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}

fn render(render_tree: &RenderTree, frame: &mut Frame, area: Rect) {
    match render_tree {
        RenderTree::Text(text) => frame.render_widget(Paragraph::new(text.clone()), area),
        RenderTree::Block(title, body) => {
            let block = Block::bordered().title(title.clone());
            render(body, frame, block.inner(area));
            frame.render_widget(block, area);
        }
        RenderTree::Stack(direction, stack_elems) => {
            let constraints = stack_elems.iter().map(|e| e.constraint).collect::<Vec<_>>();
            let layout = Layout::default()
                .direction(*direction)
                .constraints(constraints)
                .split(area);

            for (content, area) in stack_elems.iter().zip(layout.iter()) {
                render(&content.content, frame, *area);
            }
        }
    }
}
