use crate::core::layout::{Limits, Node};
use crate::core::renderer::{self, Renderer as _};
use crate::core::text::{
    Alignment as TextAlignment, LineHeight, Shaping, Span as CoreSpan, Text as CoreText, Wrapping,
};
use crate::core::widget::{Tree, tree};
use crate::core::{
    Clipboard, Element, Event, Layout, Length, Point, Rectangle, Shell, Size, Widget,
};
use crate::text;

use super::mnemonic::{get_show_underlines, parse_mnemonic};

/// Icon to display as a prefix in a menu item (e.g., checkbox checkmark).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuItemIcon {
    /// A checkmark icon (for checked checkbox items)
    Checkmark,
    /// An empty checkbox box (for unchecked checkbox items)
    CheckboxBox,
    /// No icon (empty space reserved for alignment)
    None,
}

const ELLIPSIS: &str = "…";

pub(crate) fn menu_item_line<'a, Message>(
    prefix: impl Into<String>,
    label: impl Into<String>,
    shortcut: impl Into<String>,
    suffix: impl Into<String>,
    shortcut_style: fn(&crate::Theme) -> crate::text::Style,
) -> Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
{
    Element::new(MenuItemLine::<Message>::new(
        prefix.into(),
        None,
        label.into(),
        shortcut.into(),
        suffix.into(),
        shortcut_style,
    ))
}

/// Creates a menu item line with an optional icon prefix (e.g., for checkbox items).
pub(crate) fn menu_item_line_with_icon<'a, Message>(
    prefix_icon: Option<MenuItemIcon>,
    label: impl Into<String>,
    shortcut: impl Into<String>,
    suffix: impl Into<String>,
    shortcut_style: fn(&crate::Theme) -> crate::text::Style,
) -> Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
{
    Element::new(MenuItemLine::<Message>::new(
        String::new(),
        prefix_icon,
        label.into(),
        shortcut.into(),
        suffix.into(),
        shortcut_style,
    ))
}

pub(crate) struct MenuItemLineTag;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MenuItemLineMetrics {
    pub label_column_w: f32,
    pub shortcut_w: f32,
    pub suffix_w: f32,
    pub gap: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MenuItemLineColumns {
    pub menu_width: f32,
    pub shortcut_w: f32,
    pub suffix_w: f32,
    pub gap: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MenuItemLineState {
    pub metrics: MenuItemLineMetrics,
    pub columns: Option<MenuItemLineColumns>,
}

struct MenuItemLine<Message> {
    prefix: String,
    prefix_icon: Option<MenuItemIcon>,
    label_raw: String,
    shortcut: String,
    suffix: String,
    gap: f32,

    cached_label_max_width: f32,
    cached_underline: bool,

    prefix_text: text::Text<'static, crate::Theme, crate::Renderer>,
    label_rich: text::Rich<'static, (), Message, crate::Theme, crate::Renderer>,
    shortcut_text: text::Text<'static, crate::Theme, crate::Renderer>,
    suffix_text: text::Text<'static, crate::Theme, crate::Renderer>,
}

impl<Message> MenuItemLine<Message> {
    fn new(
        prefix: String,
        prefix_icon: Option<MenuItemIcon>,
        label_raw: String,
        shortcut: String,
        suffix: String,
        shortcut_style: fn(&crate::Theme) -> crate::text::Style,
    ) -> Self {
        let prefix_text = crate::core::widget::Text::new(prefix.clone())
            .wrapping(Wrapping::None)
            .width(Length::Shrink)
            .height(Length::Shrink);

        let shortcut_text = crate::core::widget::Text::new(shortcut.clone())
            .wrapping(Wrapping::None)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .style(shortcut_style);

        let suffix_text = crate::core::widget::Text::new(suffix.clone())
            .wrapping(Wrapping::None)
            .width(Length::Shrink)
            .height(Length::Shrink);

        let underline = get_show_underlines();
        let parsed = parse_mnemonic(&label_raw);
        let display = parsed.display_text.as_ref();

        let spans: Vec<
            CoreSpan<'static, (), <crate::Renderer as crate::core::text::Renderer>::Font>,
        > = if let Some(byte_idx) = parsed.underline_index {
            let before = &display[..byte_idx];
            let mnemonic_end = display[byte_idx..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| byte_idx + i)
                .unwrap_or(display.len());
            let mnemonic = &display[byte_idx..mnemonic_end];
            let after = &display[mnemonic_end..];

            vec![
                CoreSpan::new(before.to_owned()),
                CoreSpan::new(mnemonic.to_owned()).underline(underline),
                CoreSpan::new(after.to_owned()),
            ]
        } else {
            vec![CoreSpan::new(display.to_owned())]
        };

        let this = Self {
            prefix,
            prefix_icon,
            label_raw,
            shortcut,
            suffix,
            gap: 24.0,
            cached_label_max_width: -1.0,
            cached_underline: underline,
            prefix_text,
            label_rich: text::Rich::with_spans(spans)
                .wrapping(Wrapping::None)
                .width(Length::Shrink)
                .height(Length::Shrink),
            shortcut_text,
            suffix_text,
        };

        this
    }

    fn needs_truncation(measured_width: f32, max_width: f32) -> bool {
        measured_width > max_width + 0.5
    }

    fn measure_plain_text(renderer: &crate::Renderer, content: &str, max_width: f32) -> f32 {
        use crate::core::text::Paragraph as _;
        use crate::core::text::Renderer as _;

        let paragraph =
            <crate::Renderer as crate::core::text::Renderer>::Paragraph::with_text(CoreText {
                content,
                bounds: Size::new(max_width, f32::INFINITY),
                size: renderer.default_size(),
                line_height: LineHeight::default(),
                font: renderer.default_font(),
                align_x: TextAlignment::Default,
                align_y: crate::core::alignment::Vertical::Top,
                shaping: Shaping::default(),
                wrapping: Wrapping::None,
                hint_factor: None,
            });

        paragraph.min_bounds().width
    }

    fn ellipsize(display: &str, renderer: &crate::Renderer, max_width: f32) -> String {
        if display.is_empty() {
            return String::new();
        }

        if max_width.is_infinite() {
            return display.to_owned();
        }

        let full_w = Self::measure_plain_text(renderer, display, max_width);

        if !Self::needs_truncation(full_w, max_width) {
            return display.to_owned();
        }

        let ell_w = Self::measure_plain_text(renderer, ELLIPSIS, max_width);
        if Self::needs_truncation(ell_w, max_width) {
            return ELLIPSIS.to_owned();
        }

        let mut char_offsets: Vec<usize> = display.char_indices().map(|(i, _)| i).collect();
        char_offsets.push(display.len());

        let mut lo = 0usize;
        let mut hi = char_offsets.len().saturating_sub(1);
        let mut best = 0usize;

        while lo <= hi {
            let mid = (lo + hi) / 2;
            let end = char_offsets[mid];
            let candidate = format!("{}{}", &display[..end], ELLIPSIS);
            let w = Self::measure_plain_text(renderer, &candidate, max_width);

            if !Self::needs_truncation(w, max_width) {
                best = end;
                lo = mid + 1;
            } else {
                if mid == 0 {
                    break;
                }
                hi = mid - 1;
            }
        }

        let out = format!("{}{}", &display[..best], ELLIPSIS);
        out
    }

    fn rebuild_label_rich(&mut self, renderer: &crate::Renderer, max_width: f32) {
        let underline = get_show_underlines();

        let parsed = parse_mnemonic(&self.label_raw);
        let display = parsed.display_text.as_ref();

        let truncated_display = Self::ellipsize(display, renderer, max_width);

        // Preserve underline only if the underlined character survived truncation.
        let underline_index = parsed
            .underline_index
            .and_then(|byte_idx| display.get(..byte_idx).map(|s| s.chars().count()));

        let underline_char_pos = underline_index.and_then(|pos| {
            let truncated = truncated_display != display;
            let kept_chars = truncated_display.chars().count();
            let kept_limit = if truncated {
                kept_chars.saturating_sub(1)
            } else {
                kept_chars
            };

            (pos < kept_limit).then_some(pos)
        });

        let spans: Vec<
            CoreSpan<'static, (), <crate::Renderer as crate::core::text::Renderer>::Font>,
        > = if let Some(pos) = underline_char_pos {
            let mut start_byte = None;
            let mut end_byte = None;

            for (i, (byte, ch)) in truncated_display.char_indices().enumerate() {
                if i == pos {
                    start_byte = Some(byte);
                    end_byte = Some(byte + ch.len_utf8());
                    break;
                }
            }

            if let (Some(start), Some(end)) = (start_byte, end_byte) {
                vec![
                    CoreSpan::new(truncated_display[..start].to_owned()),
                    CoreSpan::new(truncated_display[start..end].to_owned()).underline(underline),
                    CoreSpan::new(truncated_display[end..].to_owned()),
                ]
            } else {
                vec![CoreSpan::new(truncated_display.to_owned())]
            }
        } else {
            vec![CoreSpan::new(truncated_display.to_owned())]
        };

        self.label_rich = text::Rich::with_spans(spans)
            .wrapping(Wrapping::None)
            .width(Length::Shrink)
            .height(Length::Shrink);

        self.cached_label_max_width = max_width;
        self.cached_underline = underline;
    }

    fn sync(&mut self, renderer: &crate::Renderer, max_width: f32) {
        let underline = get_show_underlines();

        if (self.cached_label_max_width - max_width).abs() < 0.5
            && underline == self.cached_underline
        {
            return;
        }

        self.rebuild_label_rich(renderer, max_width);
    }
}

impl<Message> Widget<Message, crate::Theme, crate::Renderer> for MenuItemLine<Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuItemLineTag>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuItemLineState::default())
    }

    fn size(&self) -> Size<Length> {
        // Shrink so intrinsic measurement works correctly.
        // The menu layout constrains items to the computed menu width.
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.prefix_text as &dyn Widget<Message, crate::Theme, crate::Renderer>),
            Tree::new(&self.label_rich as &dyn Widget<Message, crate::Theme, crate::Renderer>),
            Tree::new(&self.shortcut_text as &dyn Widget<Message, crate::Theme, crate::Renderer>),
            Tree::new(&self.suffix_text as &dyn Widget<Message, crate::Theme, crate::Renderer>),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() != 4 {
            *tree = Tree::new(self as &dyn Widget<_, _, _>);
            return;
        }

        tree.children[0]
            .diff(&self.prefix_text as &dyn Widget<Message, crate::Theme, crate::Renderer>);
        tree.children[1]
            .diff(&self.label_rich as &dyn Widget<Message, crate::Theme, crate::Renderer>);
        tree.children[2]
            .diff(&self.shortcut_text as &dyn Widget<Message, crate::Theme, crate::Renderer>);
        tree.children[3]
            .diff(&self.suffix_text as &dyn Widget<Message, crate::Theme, crate::Renderer>);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &crate::Renderer, limits: &Limits) -> Node {
        let child_limits = limits.width(Length::Shrink).height(Length::Shrink);
        let compress_width = limits.compression().width;

        // Layout trailing pieces first so we can reserve their width.
        let prefix_node = (&mut self.prefix_text
            as &mut dyn Widget<Message, crate::Theme, crate::Renderer>)
            .layout(&mut tree.children[0], renderer, &child_limits);
        let shortcut_node = (&mut self.shortcut_text
            as &mut dyn Widget<Message, crate::Theme, crate::Renderer>)
            .layout(&mut tree.children[2], renderer, &child_limits);
        let suffix_node = (&mut self.suffix_text
            as &mut dyn Widget<Message, crate::Theme, crate::Renderer>)
            .layout(&mut tree.children[3], renderer, &child_limits);

        let prefix_size = prefix_node.size();
        let shortcut_size = shortcut_node.size();
        let suffix_size = suffix_node.size();

        // For prefix_icon, use a fixed size for the icon area (similar to checkbox size)
        const ICON_SIZE: f32 = 16.0;
        const PREFIX_GAP: f32 = 6.0; // Gap between prefix icon and label
        let (prefix_w, prefix_gap) = if self.prefix_icon.is_some() {
            (ICON_SIZE, PREFIX_GAP)
        } else if self.prefix.is_empty() {
            (0.0, 0.0)
        } else {
            (prefix_size.width, 0.0)
        };
        let shortcut_w = if self.shortcut.is_empty() {
            0.0
        } else {
            shortcut_size.width
        };
        let suffix_w = if self.suffix.is_empty() {
            0.0
        } else {
            suffix_size.width
        };

        let has_trailing = shortcut_w > 0.0 || suffix_w > 0.0;

        let trailing_block_w = match (shortcut_w > 0.0, suffix_w > 0.0) {
            (true, true) => shortcut_w + self.gap + suffix_w,
            (true, false) => shortcut_w,
            (false, true) => suffix_w,
            (false, false) => 0.0,
        };

        let max_width = limits.max().width;

        let columns = tree
            .state
            .downcast_ref::<MenuItemLineState>()
            .columns
            .unwrap_or_default();

        let reserved_trailing_w = if !compress_width && max_width.is_finite() {
            let has_shortcut_col = columns.shortcut_w > 0.0;
            let has_suffix_col = columns.suffix_w > 0.0;

            let mut w = 0.0;
            if has_shortcut_col {
                w += columns.shortcut_w;
            }
            if has_suffix_col {
                if has_shortcut_col {
                    w += columns.gap;
                }
                w += columns.suffix_w;
            }

            if (has_shortcut_col || has_suffix_col) && w > 0.0 {
                w + columns.gap
            } else {
                0.0
            }
        } else {
            0.0
        };

        // In compression (intrinsic measurement) passes we must not ellipsize, otherwise the
        // measured width becomes self-fulfilling and the menu can never grow to fit.
        let max_width_for_truncation = if compress_width {
            f32::INFINITY
        } else {
            max_width
        };

        let label_max = if max_width_for_truncation.is_finite() {
            let available_for_label = max_width_for_truncation
                - prefix_w
                - prefix_gap
                - if reserved_trailing_w > 0.0 {
                    reserved_trailing_w
                } else {
                    trailing_block_w + if has_trailing { self.gap } else { 0.0 }
                };
            available_for_label.max(0.0)
        } else {
            f32::INFINITY
        };

        self.sync(renderer, label_max);

        let label_limits = child_limits.max_width(label_max);
        let label_node = (&mut self.label_rich
            as &mut dyn Widget<Message, crate::Theme, crate::Renderer>)
            .layout(&mut tree.children[1], renderer, &label_limits);
        let label_size = label_node.size();

        // For icon prefix, use the icon size for height calculation
        let prefix_height = if self.prefix_icon.is_some() {
            ICON_SIZE
        } else {
            prefix_size.height
        };

        let max_h = prefix_height
            .max(label_size.height)
            .max(shortcut_size.height)
            .max(suffix_size.height);

        // Decide final width:
        // - If columns are set, we're in pass 2: use max_width (which is the inner width after parent padding)
        // - Otherwise use intrinsic for measurement, max_width for rendering
        let prefix_total_w = prefix_w + prefix_gap;
        let intrinsic_width = prefix_total_w
            + label_size.width
            + if has_trailing { self.gap } else { 0.0 }
            + trailing_block_w;

        let total_w = if columns.menu_width > 0.0 && max_width.is_finite() {
            // Columns are set and we have a finite max_width: use it (accounts for parent padding)
            max_width
        } else if compress_width || !max_width.is_finite() {
            intrinsic_width
        } else {
            max_width
        };

        {
            let state = tree.state.downcast_mut::<MenuItemLineState>();
            state.metrics = MenuItemLineMetrics {
                label_column_w: prefix_total_w + label_size.width,
                shortcut_w,
                suffix_w,
                gap: self.gap,
            };
        }

        let mut children = Vec::with_capacity(4);

        // Prefix and label are left-aligned
        // For icon prefix, create a node with the icon size
        let prefix_layout_node = if self.prefix_icon.is_some() {
            Node::new(Size::new(ICON_SIZE, max_h))
        } else {
            prefix_node.move_to(Point::new(0.0, (max_h - prefix_size.height) / 2.0))
        };
        children.push(prefix_layout_node);
        children.push(label_node.move_to(Point::new(
            prefix_total_w,
            (max_h - label_size.height) / 2.0,
        )));

        // Trailing parts are right-aligned in a shared column.
        // Shortcut and suffix share the same column (an item has one or the other).
        let layout_width = total_w;

        if columns.shortcut_w > 0.0 || columns.suffix_w > 0.0 {
            // We have column info: place trailing content at the right edge.
            // Both shortcut and suffix are right-aligned in the same column.
            let shortcut_x = (layout_width - shortcut_w).max(0.0);
            let suffix_x = (layout_width - suffix_w).max(0.0);

            children.push(
                shortcut_node.move_to(Point::new(shortcut_x, (max_h - shortcut_size.height) / 2.0)),
            );
            children.push(
                suffix_node.move_to(Point::new(suffix_x, (max_h - suffix_size.height) / 2.0)),
            );
        } else {
            // No column info yet: place trailing parts after label (intrinsic positioning).
            let trailing_start_x =
                prefix_total_w + label_size.width + if has_trailing { self.gap } else { 0.0 };
            let shortcut_x = trailing_start_x;
            let suffix_x = match (shortcut_w > 0.0, suffix_w > 0.0) {
                (true, true) => trailing_start_x + shortcut_w + self.gap,
                (false, true) => trailing_start_x,
                _ => trailing_start_x,
            };

            children.push(
                shortcut_node.move_to(Point::new(shortcut_x, (max_h - shortcut_size.height) / 2.0)),
            );
            children.push(
                suffix_node.move_to(Point::new(suffix_x, (max_h - suffix_size.height) / 2.0)),
            );
        }

        Node::with_children(Size::new(total_w, max_h), children)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: crate::core::mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();

        (&mut self.prefix_text as &mut dyn Widget<Message, crate::Theme, crate::Renderer>).update(
            &mut tree.children[0],
            event,
            children.next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        (&mut self.label_rich as &mut dyn Widget<Message, crate::Theme, crate::Renderer>).update(
            &mut tree.children[1],
            event,
            children.next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        (&mut self.shortcut_text as &mut dyn Widget<Message, crate::Theme, crate::Renderer>)
            .update(
                &mut tree.children[2],
                event,
                children.next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

        (&mut self.suffix_text as &mut dyn Widget<Message, crate::Theme, crate::Renderer>).update(
            &mut tree.children[3],
            event,
            children.next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: crate::core::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use crate::core::text::Renderer as TextRenderer;

        let mut children = layout.children();
        let prefix_layout = children.next().unwrap();

        // Draw prefix: either as icon (if prefix_icon is set) or as text
        if let Some(icon) = self.prefix_icon {
            let bounds = prefix_layout.bounds();
            match icon {
                MenuItemIcon::Checkmark => {
                    // Draw checkmark icon using icon font
                    let icon_char = <crate::Renderer as TextRenderer>::CHECKMARK_ICON;
                    let icon_font = <crate::Renderer as TextRenderer>::ICON_FONT;
                    let size = crate::core::Pixels(bounds.height * 0.7);

                    renderer.fill_text(
                        crate::core::text::Text {
                            content: icon_char.to_string(),
                            font: icon_font,
                            size,
                            line_height: LineHeight::default(),
                            bounds: bounds.size(),
                            align_x: TextAlignment::Center,
                            align_y: crate::core::alignment::Vertical::Center,
                            shaping: Shaping::Basic,
                            wrapping: Wrapping::None,
                            hint_factor: None,
                        },
                        bounds.center(),
                        style.text_color,
                        *viewport,
                    );
                }
                MenuItemIcon::CheckboxBox => {
                    // Draw an empty checkbox box (like unchecked checkbox widget)
                    let box_size = (bounds.height * 0.7).min(bounds.width);
                    let box_bounds = Rectangle {
                        x: bounds.x + (bounds.width - box_size) / 2.0,
                        y: bounds.y + (bounds.height - box_size) / 2.0,
                        width: box_size,
                        height: box_size,
                    };

                    // Use theme colors similar to checkbox
                    let border_color = theme.background.divider;
                    let background_color = theme.background.base;

                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: box_bounds,
                            border: crate::core::Border {
                                radius: 2.0.into(),
                                width: 1.0,
                                color: border_color,
                            },
                            ..renderer::Quad::default()
                        },
                        crate::core::Background::Color(background_color),
                    );
                }
                MenuItemIcon::None => {
                    // Draw nothing, space is reserved for alignment
                }
            }
        } else {
            // Draw prefix as regular text
            (&self.prefix_text as &dyn Widget<Message, crate::Theme, crate::Renderer>).draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                prefix_layout,
                cursor,
                viewport,
            );
        }

        // Clip label to its own bounds so it never bleeds into the shortcut/suffix area.
        let label_layout = children.next().unwrap();
        let label_viewport = label_layout
            .bounds()
            .intersection(viewport)
            .unwrap_or(*viewport);

        (&self.label_rich as &dyn Widget<Message, crate::Theme, crate::Renderer>).draw(
            &tree.children[1],
            renderer,
            theme,
            style,
            label_layout,
            cursor,
            &label_viewport,
        );

        (&self.shortcut_text as &dyn Widget<Message, crate::Theme, crate::Renderer>).draw(
            &tree.children[2],
            renderer,
            theme,
            style,
            children.next().unwrap(),
            cursor,
            viewport,
        );

        (&self.suffix_text as &dyn Widget<Message, crate::Theme, crate::Renderer>).draw(
            &tree.children[3],
            renderer,
            theme,
            style,
            children.next().unwrap(),
            cursor,
            viewport,
        );
    }
}
