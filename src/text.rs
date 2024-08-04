use ratatui::text::Text;
use topogi_lang::ast::Exp;

use crate::render_tree::RenderTree;
use crate::render_tree::Result;

pub fn create_text(exp: &Exp) -> Result<RenderTree> {
    Ok(RenderTree::Text(Text::raw(exp.to_string())))
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
            create_text(&exp),
            Ok(RenderTree::Text(Text::raw("hello world")))
        );
    }
}
