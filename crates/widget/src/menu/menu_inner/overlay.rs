// From iced_aw, license MIT
// Ported from libcosmic

//! Overlay trait implementation for Menu

use super::menu::Menu;
use crate::core::layout::{Layout, Limits, Node};
use crate::core::mouse::Cursor;
use crate::core::{Clipboard, Shell, Size, event, overlay, renderer};
use crate::menu::style::StyleSheet;

impl<'a, 'b, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Menu<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        Menu::layout(
            self,
            renderer,
            Limits::NONE
                .min_width(bounds.width)
                .max_width(bounds.width)
                .min_height(bounds.height)
                .max_height(bounds.height),
        )
    }

    fn update(
        &mut self,
        event: &event::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let _ = self.on_event_inner(event, layout, cursor, renderer, clipboard, shell);
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        self.draw(renderer, theme, style, layout, cursor);
    }
}
