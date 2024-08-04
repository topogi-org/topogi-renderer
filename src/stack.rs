use ratatui::layout::{Constraint, Direction};
use topogi_lang::ast::Exp;

use crate::render_tree::{
    check_symbol, create_integer, create_list_with_len, create_list_with_minlen,
    create_render_tree, RenderTree, RenderTreeError, Result,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackElement<'a> {
    pub constraint: Constraint,
    pub content: Box<RenderTree<'a>>,
}

impl<'a> StackElement<'a> {
    pub fn new(constraint: Constraint, content: RenderTree<'a>) -> Self {
        StackElement {
            constraint,
            content: Box::new(content),
        }
    }
}

fn create_constraint(exp: &Exp) -> Result<Constraint> {
    let elems = create_list_with_len(exp, 2)?;

    let kind = elems[0].as_symbol().ok_or(RenderTreeError::ExpectedSymbol(
        "constraint kind",
        exp.clone(),
    ))?;

    match kind {
        "length" => {
            let value = create_integer(&elems[1])?;
            Ok(Constraint::Length(value as u16))
        }
        "min" => {
            let value = create_integer(&elems[1])?;
            Ok(Constraint::Min(value as u16))
        }
        "max" => {
            let value = create_integer(&elems[1])?;
            Ok(Constraint::Max(value as u16))
        }
        "percentage" => {
            let value = create_integer(&elems[1])?;
            Ok(Constraint::Percentage(value as u16))
        }
        "fill" => {
            let value = create_integer(&elems[1])?;
            Ok(Constraint::Fill(value as u16))
        }
        _ => Err(RenderTreeError::ExpectedSymbol(
            "constraint kind",
            exp.clone(),
        )),
    }
}

fn create_stack_element(exp: &Exp) -> Result<StackElement> {
    let elems = create_list_with_len(exp, 2)?;

    let constraint = create_constraint(&elems[0])?;
    let content = create_render_tree(&elems[1])?;

    Ok(StackElement::new(constraint, content))
}

fn create_direction(exp: &Exp) -> Result<Direction> {
    let direction = exp.as_symbol().ok_or(RenderTreeError::ExpectedSymbol(
        "horizontal or vertical",
        exp.clone(),
    ))?;
    match direction {
        "horizontal" => Ok(Direction::Horizontal),
        "vertical" => Ok(Direction::Vertical),
        _ => Err(RenderTreeError::InvalidDirection(direction.to_string())),
    }
}

pub fn create_stack(exp: &Exp) -> Result<RenderTree> {
    let elems = create_list_with_minlen(exp, 3)?;
    check_symbol(&elems[0], "stack")?;

    let direction = create_direction(&elems[1])?;

    let stack_elements = elems
        .iter()
        .skip(2)
        .map(|e| create_stack_element(e))
        .collect::<Result<Vec<StackElement>>>()?;

    Ok(RenderTree::Stack(direction, stack_elements))
}

#[cfg(test)]
mod tests {
    use ratatui::{text::Text, widgets::Block};

    use super::*;
    fn parse(str: &str) -> Exp {
        let mut parser = topogi_lang::parser::Parser::new(str);
        parser.parse_exp().unwrap()
    }

    #[test]
    fn test_create_constraint() {
        let exp = parse(r#"(length 3)"#);
        assert_eq!(create_constraint(&exp), Ok(Constraint::Length(3)));
    }

    #[test]
    fn test_create_stack_element() {
        let exp = parse(r#"((length 3) (block "title" "content"))"#);
        assert_eq!(
            create_stack_element(&exp),
            Ok(StackElement::new(
                Constraint::Length(3),
                RenderTree::Block(
                    Block::default().title("title"),
                    Box::new(RenderTree::Text(Text::raw("content")))
                )
            ))
        );
    }

    #[test]
    fn test_create_stack() {
        let exp = parse(
            r#"(stack horizontal
                        ((length 3) (block "title1" "content1"))
                        ((length 3) (block "title2" "content2"))
                   )"#,
        );
        assert_eq!(
            create_stack(&exp),
            Ok(RenderTree::Stack(
                Direction::Horizontal,
                vec![
                    StackElement::new(
                        Constraint::Length(3),
                        RenderTree::Block(
                            Block::default().title("title1"),
                            Box::new(RenderTree::Text(Text::raw("content1")))
                        )
                    ),
                    StackElement::new(
                        Constraint::Length(3),
                        RenderTree::Block(
                            Block::default().title("title2"),
                            Box::new(RenderTree::Text(Text::raw("content2")))
                        )
                    )
                ]
            ))
        );
    }
}
