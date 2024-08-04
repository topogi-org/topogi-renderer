use ratatui::layout::{Alignment, Constraint, Direction};
use topogi_lang::ast::Exp;

type Result<T> = std::result::Result<T, RenderTreeError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RenderLayer {
    trees: Vec<RenderTree>,
}

impl RenderLayer {
    pub fn new() -> Self {
        RenderLayer { trees: Vec::new() }
    }

    pub fn add_layer(&mut self, tree: RenderTree) {
        self.trees.push(tree);
    }

    pub fn iter(&self) -> std::slice::Iter<RenderTree> {
        self.trees.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RenderTree {
    // (text)
    Text(String),
    Block(Block),
    // (stack direction (constraint content)*)
    Stack(Direction, Vec<StackElement>),
}

impl RenderTree {
    pub fn block(title: String, content: RenderTree) -> Self {
        RenderTree::Block(Block::new(title, content))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub title: String,
    pub content: Box<RenderTree>,
    pub style: BlockStyle,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BlockStyle {
    pub title_align: Alignment,
}

impl BlockStyle {
    pub fn from_exp(exp: &Exp) -> Result<Self> {
        let mut block_style = BlockStyle::default();
        let elems = exp
            .as_list()
            .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

        if elems.len() < 2 {
            return Err(RenderTreeError::InvalidLength(exp.clone()));
        }

        if elems[0].as_symbol() != Some("style") {
            return Err(RenderTreeError::ExpectedSymbol("style", exp.clone()));
        }

        for style in elems.iter().skip(1) {
            if let Ok(align) = BlockStyle::title_align(style) {
                block_style.title_align = align;
            }
        }

        Ok(block_style)
    }

    fn title_align(exp: &Exp) -> Result<Alignment> {
        let elems = exp
            .as_list()
            .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;
        if elems.len() != 2 {
            return Err(RenderTreeError::InvalidLength(exp.clone()));
        }
        if elems[0].as_symbol() != Some("title_align") {
            return Err(RenderTreeError::ExpectedSymbol("title_align", exp.clone()));
        }

        match elems[1].as_symbol() {
            Some("center") => Ok(Alignment::Center),
            Some("left") => Ok(Alignment::Left),
            Some("right") => Ok(Alignment::Right),
            _ => Err(RenderTreeError::ExpectedSymbol(
                "center | left | right",
                exp.clone(),
            )),
        }
    }
}

impl Block {
    pub fn new(title: String, content: RenderTree) -> Self {
        Block {
            title,
            content: Box::new(content),
            style: BlockStyle::default(),
        }
    }

    pub fn from_exp(exp: &Exp) -> Result<Self> {
        let elems = exp
            .as_list()
            .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

        if elems.len() < 3 {
            return Err(RenderTreeError::InvalidLength(exp.clone()));
        }

        if elems[0].as_symbol() != Some("block") {
            return Err(RenderTreeError::ExpectedSymbol("block", exp.clone()));
        }

        let mut block = Block::new(elems[1].to_string(), create_render_tree(&elems[2])?);
        if let Some(style) = elems.get(3) {
            block.style = BlockStyle::from_exp(style)?;
        }

        Ok(block)
    }
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
    Ok(RenderTree::Text(exp.to_string()))
}

fn create_integer(exp: &Exp) -> Result<i64> {
    exp.as_integer()
        .ok_or(RenderTreeError::ExpectInteger(exp.clone()))
}

fn create_constraint(exp: &Exp) -> Result<Constraint> {
    let elems = exp
        .as_list()
        .ok_or(RenderTreeError::ExpectedList(exp.clone()))?;

    if elems.len() != 2 {
        return Err(RenderTreeError::ExpectedList(exp.clone()));
    }

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
    Block::from_exp(exp)
        .map(|block| RenderTree::Block(block))
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
            Block::from_exp(&exp),
            Ok(Block {
                title: "title".to_string(),
                content: Box::new(RenderTree::Text("content".to_string())),
                style: BlockStyle::default()
            })
        );

        let exp = parse(r#"(block title context (style (title_align center)))"#);
        assert_eq!(
            Block::from_exp(&exp),
            Ok(Block {
                title: "title".to_string(),
                content: Box::new(RenderTree::Text("context".to_string())),
                style: BlockStyle {
                    title_align: Alignment::Center
                }
            })
        );
    }

    #[test]
    fn test_create_nested_block() {
        let exp = parse(r#"(block "title" (block "title2" "content"))"#);
        assert_eq!(
            create_render_tree(&exp),
            Ok(RenderTree::block(
                "title".to_string(),
                RenderTree::block(
                    "title2".to_string(),
                    RenderTree::Text("content".to_string())
                )
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
                RenderTree::block("title".to_string(), RenderTree::Text("content".to_string()))
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
                        RenderTree::block(
                            "title1".to_string(),
                            RenderTree::Text("content1".to_string())
                        )
                    ),
                    StackElement::new(
                        Constraint::Length(3),
                        RenderTree::block(
                            "title2".to_string(),
                            RenderTree::Text("content2".to_string())
                        )
                    )
                ]
            ))
        )
    }

    #[test]
    fn test_layer() {
        let exp = parse(
            r#"(layer
                    (block "title1" "content1")
                    (stack horizontal
                        ((length 3) (block "title2" "content2"))
                    )
               )"#,
        );
        assert_eq!(
            create_render_layer(&exp),
            Ok(RenderLayer {
                trees: vec![
                    RenderTree::block(
                        "title1".to_string(),
                        RenderTree::Text("content1".to_string())
                    ),
                    RenderTree::Stack(
                        Direction::Horizontal,
                        vec![StackElement::new(
                            Constraint::Length(3),
                            RenderTree::block(
                                "title2".to_string(),
                                RenderTree::Text("content2".to_string())
                            )
                        )]
                    )
                ]
            })
        )
    }
}
