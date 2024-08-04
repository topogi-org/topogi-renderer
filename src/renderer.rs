use ratatui::{
    layout::{Layout, Rect},
    Frame,
};

use crate::render_tree::{RenderLayer, RenderTree};

pub fn render_tree(tree: &RenderTree, frame: &mut Frame, area: Rect) {
    match tree {
        RenderTree::Text(text) => frame.render_widget(text, area),
        RenderTree::Block(block, content) => {
            render_tree(content, frame, block.inner(area));
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

pub fn render_layer(layer: &RenderLayer, frame: &mut Frame, area: Rect) {
    for tree in layer.iter() {
        render_tree(tree, frame, area);
    }
}
