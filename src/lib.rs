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
use render_tree::{create_render_layer, create_render_tree, RenderLayer, RenderTree};
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

    pub fn render_layer(&mut self, exp: &Exp) -> Result<()> {
        let layer = create_render_layer(exp).map_err(RenderError::RenderTreeError)?;
        self.terminal.draw(|frame| {
            let area = frame.size();
            render_layer(&layer, frame, area);
        })?;
        Ok(())
    }

    pub fn render(&mut self, exp: &Exp) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.size();
            match create_render_tree(exp) {
                Ok(tree) => render_tree(&tree, frame, area),
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

fn render_tree(tree: &RenderTree, frame: &mut Frame, area: Rect) {
    match tree {
        RenderTree::Text(text) => frame.render_widget(Paragraph::new(text.clone()), area),
        RenderTree::Block(title, body) => {
            let block = Block::bordered().title(title.clone());
            render_tree(body, frame, block.inner(area));
            frame.render_widget(block, area);
        }
        RenderTree::Stack(direction, stack_elems) => {
            let constraints = stack_elems.iter().map(|e| e.constraint).collect::<Vec<_>>();
            let layout = Layout::default()
                .direction(*direction)
                .constraints(constraints)
                .split(area);

            for (content, area) in stack_elems.iter().zip(layout.iter()) {
                render_tree(&content.content, frame, *area);
            }
        }
    }
}

fn render_layer(layer: &RenderLayer, frame: &mut Frame, area: Rect) {
    for tree in layer.iter() {
        render_tree(tree, frame, area);
    }
}
