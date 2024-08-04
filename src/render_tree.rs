use ratatui::{layout::Direction, text::Text, widgets::Block};
use topogi_lang::ast::Exp;

use crate::{
    block::create_block,
    stack::{create_stack, StackElement},
    text::create_text,
};

pub type Result<T> = std::result::Result<T, RenderTreeError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RenderTree<'a> {
    Text(Text<'a>),
    Block(Block<'a>, Box<RenderTree<'a>>),
    Stack(Direction, Vec<StackElement<'a>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RenderLayer<'a> {
    trees: Vec<RenderTree<'a>>,
}

impl<'a> RenderLayer<'a> {
    pub fn new() -> Self {
        RenderLayer { trees: Vec::new() }
    }

    pub fn add_layer(&mut self, tree: RenderTree<'a>) {
        self.trees.push(tree);
    }

    pub fn iter(&self) -> std::slice::Iter<RenderTree> {
        self.trees.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RenderTreeError {
    ExpectedList(Exp),
    ExpectInteger(Exp),
    ExpectedSymbol(&'static str, Exp),
    ExpectedString(Exp),
    InvalidLength(Exp),
    InvalidDirection(String),
}

pub fn create_integer(exp: &Exp) -> Result<i64> {
    exp.as_integer()
        .ok_or(RenderTreeError::ExpectInteger(exp.clone()))
}

pub fn create_list_with_len(exp: &Exp, len: usize) -> Result<&[Exp]> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;
    if elems.len() != len {
        return Err(RenderTreeError::InvalidLength(exp.clone()));
    }
    Ok(elems)
}

pub fn create_list_with_minlen(exp: &Exp, minlen: usize) -> Result<&[Exp]> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;
    if elems.len() < minlen {
        return Err(RenderTreeError::InvalidLength(exp.clone()));
    }
    Ok(elems)
}

pub fn check_symbol(exp: &Exp, expected: &'static str) -> Result<()> {
    if exp.as_symbol() != Some(expected) {
        return Err(RenderTreeError::ExpectedSymbol(expected, exp.clone()));
    }
    Ok(())
}

pub fn create_render_tree(exp: &Exp) -> Result<RenderTree> {
    create_block(exp)
        .or_else(|_| create_stack(exp))
        .or_else(|_| create_text(exp))
}

pub fn create_render_layer(exp: &Exp) -> Result<RenderLayer> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

    if elems[0].as_symbol() != Some("layer") {
        return Err(RenderTreeError::ExpectedSymbol("layer", exp.clone()));
    }

    let trees = elems
        .iter()
        .skip(1)
        .map(|e| create_render_tree(e))
        .collect::<Result<_>>()?;

    Ok(RenderLayer { trees })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Constraint;

    fn parse(str: &str) -> Exp {
        let mut parser = topogi_lang::parser::Parser::new(str);
        parser.parse_exp().unwrap()
    }

    #[test]
    fn test_layer() {
        let exp = parse(
            r#"(layer
                 (block "title1" "content1")
                 (stack horizontal
                   ((length 3) (block "title2" "content2"))
                 ))"#,
        );
        assert_eq!(
            create_render_layer(&exp),
            Ok(RenderLayer {
                trees: vec![
                    RenderTree::Block(
                        Block::default().title("title1"),
                        Box::new(RenderTree::Text(Text::raw("content1")))
                    ),
                    RenderTree::Stack(
                        Direction::Horizontal,
                        vec![StackElement::new(
                            Constraint::Length(3),
                            RenderTree::Block(
                                Block::default().title("title2"),
                                Box::new(RenderTree::Text(Text::raw("content2")))
                            )
                        )]
                    )
                ]
            })
        );
    }
}
