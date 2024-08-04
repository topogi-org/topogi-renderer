pub mod block;
pub mod render_tree;
pub mod renderer;
pub mod stack;
pub mod text;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};
use render_tree::create_render_layer;
use renderer::render_layer;
use std::io::{stdout, Stdout};
use topogi_lang::ast::Exp;

#[derive(Debug)]
pub struct UIEngine {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Debug)]
pub enum RenderError {
    RenderTreeError(render_tree::RenderTreeError),
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
        let layer = create_render_layer(exp).map_err(RenderError::RenderTreeError)?;
        self.terminal.draw(|frame| {
            let area = frame.size();
            render_layer(&layer, frame, area);
        })?;
        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}
