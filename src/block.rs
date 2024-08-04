use crate::render_tree::{
    check_symbol, create_list_with_len, create_list_with_minlen, create_render_tree, RenderTree,
    RenderTreeError, Result,
};
use ratatui::{
    layout::Alignment,
    widgets::{Block, Borders},
};
use topogi_lang::ast::Exp;

pub fn create_block(exp: &Exp) -> Result<RenderTree> {
    let elems = create_list_with_minlen(exp, 3)?;
    check_symbol(&elems[0], "block")?;

    let mut block = Block::new().title(elems[1].to_string());
    let inner = create_render_tree(&elems[2])?;
    if let Some(style) = elems.get(3) {
        block = block_style(block.clone(), style)?;
    }

    Ok(RenderTree::Block(block, Box::new(inner)))
}

pub fn block_style<'a>(mut block: Block<'a>, exp: &Exp) -> Result<Block<'a>> {
    let elems = create_list_with_minlen(exp, 2)?;
    check_symbol(&elems[0], "style")?;

    for style in elems.iter().skip(1) {
        if let Ok(align) = title_align(style) {
            block = block.title_alignment(align);
        }

        if let Ok(borders) = borders(style) {
            block = block.borders(borders);
        }
    }

    Ok(block)
}

fn title_align(exp: &Exp) -> Result<Alignment> {
    let elems = create_list_with_len(exp, 2)?;
    check_symbol(&elems[0], "title-align")?;

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

fn borders(exp: &Exp) -> Result<Borders> {
    let elems = create_list_with_len(exp, 2)?;
    check_symbol(&elems[0], "border")?;

    match elems[1].as_symbol() {
        Some("none") => Ok(Borders::NONE),
        Some("left") => Ok(Borders::LEFT),
        Some("right") => Ok(Borders::RIGHT),
        Some("top") => Ok(Borders::TOP),
        Some("bottom") => Ok(Borders::BOTTOM),
        Some("all") => Ok(Borders::ALL),
        _ => Err(RenderTreeError::ExpectedSymbol(
            "none | left | right | top | bottom | all",
            exp.clone(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Text;
    fn parse(str: &str) -> Exp {
        let mut parser = topogi_lang::parser::Parser::new(str);
        parser.parse_exp().unwrap()
    }

    #[test]
    fn test_create_block() {
        let exp = parse(r#"(block "title" "content")"#);
        assert_eq!(
            create_block(&exp),
            Ok(RenderTree::Block(
                Block::new().title("title"),
                Box::new(RenderTree::Text(Text::raw("content")))
            ))
        );

        let exp = parse(r#"(block title context (style (title_align center)))"#);
        assert_eq!(
            create_block(&exp),
            Ok(RenderTree::Block(
                Block::new()
                    .title("title")
                    .title_alignment(Alignment::Center),
                Box::new(RenderTree::Text(Text::raw("context")))
            ))
        );
    }

    #[test]
    fn test_create_nested_block() {
        let exp = parse(r#"(block "title" (block "title2" "content"))"#);
        assert_eq!(
            create_block(&exp),
            Ok(RenderTree::Block(
                Block::new().title("title"),
                Box::new(RenderTree::Block(
                    Block::new().title("title2"),
                    Box::new(RenderTree::Text(Text::raw("content")))
                ))
            ))
        );
    }
}
