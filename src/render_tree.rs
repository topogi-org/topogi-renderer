use ratatui::layout::{Constraint, Direction};
use topogi_lang::ast::Exp;

type Result<T> = std::result::Result<T, RenderTreeError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RenderTree {
    // (text)
    Text(String),
    // (block title content)
    Block(String, Box<RenderTree>),
    // (stack direction (constraint content)*)
    Stack(Direction, Vec<StackElement>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackElement {
    pub constraint: Constraint,
    pub content: Box<RenderTree>,
}

impl StackElement {
    pub fn new(constraint: Constraint, content: RenderTree) -> Self {
        StackElement {
            constraint,
            content: Box::new(content),
        }
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

fn create_text(exp: &Exp) -> Result<RenderTree> {
    let str = exp
        .as_string()
        .ok_or(RenderTreeError::ExpectedString(exp.clone()))?;

    Ok(RenderTree::Text(str.to_string()))
}

fn create_block(exp: &Exp) -> Result<RenderTree> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

    if elems.len() != 3 {
        return Err(RenderTreeError::InvalidLength(exp.clone()));
    }

    if elems[0].as_symbol() != Some("block") {
        return Err(RenderTreeError::ExpectedSymbol("block", exp.clone()));
    }

    let title = elems[1]
        .as_string()
        .ok_or(RenderTreeError::ExpectedString(exp.clone()))?;

    let content = &elems[2];

    Ok(RenderTree::Block(
        title.to_string(),
        Box::new(create_render_tree(content)?),
    ))
}

fn create_constraint(exp: &Exp) -> Result<Constraint> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

    if elems.len() != 2 {
        return Err(RenderTreeError::ExpectedList(exp.clone()));
    }

    let _ = elems[0].as_symbol().ok_or(RenderTreeError::ExpectedSymbol(
        "constraint kind",
        exp.clone(),
    ))?;

    let length = elems[1]
        .as_integer()
        .ok_or(RenderTreeError::ExpectInteger(exp.clone()))?;

    Ok(Constraint::Length(length as u16))
}

fn create_stack_element(exp: &Exp) -> Result<StackElement> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

    if elems.len() != 2 {
        return Err(RenderTreeError::InvalidLength(exp.clone()));
    }

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

fn create_stack(exp: &Exp) -> Result<RenderTree> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

    if elems.len() < 3 {
        return Err(RenderTreeError::InvalidLength(exp.clone()));
    }

    if elems[0].as_symbol() != Some("stack") {
        return Err(RenderTreeError::ExpectedSymbol("stack", exp.clone()));
    }

    let direction = create_direction(&elems[1])?;

    let stack_elements = elems
        .iter()
        .skip(2)
        .map(|e| create_stack_element(e))
        .collect::<Result<Vec<StackElement>>>()?;

    Ok(RenderTree::Stack(direction, stack_elements))
}

pub fn create_render_tree(exp: &Exp) -> Result<RenderTree> {
    create_block(exp)
        .or_else(|_| create_stack(exp))
        .or_else(|_| create_text(exp))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(str: &str) -> Exp {
        let mut parser = topogi_lang::parser::Parser::new(str);
        parser.parse_exp().unwrap()
    }

    #[test]
    fn test_create_text() {
        let exp = parse(r#""hello world""#);
        assert_eq!(
            create_render_tree(&exp),
            Ok(RenderTree::Text("hello world".to_string()))
        );
    }

    #[test]
    fn test_create_block() {
        let exp = parse(r#"(block "title" "content")"#);
        assert_eq!(
            create_render_tree(&exp),
            Ok(RenderTree::Block(
                "title".to_string(),
                Box::new(RenderTree::Text("content".to_string()))
            ))
        );
    }

    #[test]
    fn test_create_nested_block() {
        let exp = parse(r#"(block "title" (block "title2" "content"))"#);
        assert_eq!(
            create_render_tree(&exp),
            Ok(RenderTree::Block(
                "title".to_string(),
                Box::new(RenderTree::Block(
                    "title2".to_string(),
                    Box::new(RenderTree::Text("content".to_string()))
                ))
            ))
        );
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
                    "title".to_string(),
                    Box::new(RenderTree::Text("content".to_string()))
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
            create_render_tree(&exp),
            Ok(RenderTree::Stack(
                Direction::Horizontal,
                vec![
                    StackElement::new(
                        Constraint::Length(3),
                        RenderTree::Block(
                            "title1".to_string(),
                            Box::new(RenderTree::Text("content1".to_string()))
                        )
                    ),
                    StackElement::new(
                        Constraint::Length(3),
                        RenderTree::Block(
                            "title2".to_string(),
                            Box::new(RenderTree::Text("content2".to_string()))
                        )
                    )
                ]
            ))
        )
    }
}
