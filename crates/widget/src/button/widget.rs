//! Buttons allow your users to perform actions by pressing them.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! use icy_ui::widget::button;
//!
//! #[derive(Clone)]
//! enum Message {
//!     ButtonPressed,
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     button("Press me!").on_press(Message::ButtonPressed).into()
//! }
//! ```
use crate::core::border::{self, Border};
use crate::core::keyboard;
use crate::core::keyboard::key::{self, Key};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::theme;
use crate::core::touch;
use crate::core::widget::Id as WidgetId;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Background, Clipboard, Color, Element, Event, Layout, Length, Padding, Rectangle, Shadow,
    Shell, Size, Theme, Vector, Widget,
};
use crate::focus::FocusRing;

/// A generic widget that produces a message when pressed.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use icy_ui_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
/// use icy_ui::widget::button;
///
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     button("Press me!").on_press(Message::ButtonPressed).into()
/// }
/// ```
///
/// If a [`Button::on_press`] handler is not set, the resulting [`Button`] will
/// be disabled:
///
/// ```no_run
/// # mod iced { pub mod widget { pub use icy_ui_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
/// use icy_ui::widget::button;
///
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     button("I am disabled!").into()
/// }
/// ```
pub struct Button<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: Catalog,
{
    id: Option<WidgetId>,
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<OnPress<'a, Message>>,
    on_press_down: Option<OnPress<'a, Message>>,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    selected: bool,
    class: Theme::Class<'a>,
    status: Option<Status>,
}

enum OnPress<'a, Message> {
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

impl<Message: Clone> OnPress<'_, Message> {
    fn get(&self) -> Message {
        match self {
            OnPress::Direct(message) => message.clone(),
            OnPress::Closure(f) => f(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: Catalog,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Button {
            id: None,
            content,
            on_press: None,
            on_press_down: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: DEFAULT_PADDING,
            clip: false,
            selected: false,
            class: Theme::default(),
            status: None,
        }
    }

    /// Sets the unique identifier of the [`Button`].
    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Button`] will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(OnPress::Direct(on_press));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// This is analogous to [`Button::on_press`], but using a closure to produce
    /// the message.
    ///
    /// This closure will only be called when the [`Button`] is actually pressed and,
    /// therefore, this method is useful to reduce overhead if creating the resulting
    /// message is slow.
    pub fn on_press_with(mut self, on_press: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnPress::Closure(Box::new(on_press)));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press.map(OnPress::Direct);
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed down
    /// (before release).
    ///
    /// This is useful for drag operations or hold actions.
    pub fn on_press_down(mut self, on_press_down: Message) -> Self {
        self.on_press_down = Some(OnPress::Direct(on_press_down));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed down,
    /// using a closure.
    pub fn on_press_down_with(mut self, on_press_down: impl Fn() -> Message + 'a) -> Self {
        self.on_press_down = Some(OnPress::Closure(Box::new(on_press_down)));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed down,
    /// if `Some`.
    pub fn on_press_down_maybe(mut self, on_press_down: Option<Message>) -> Self {
        self.on_press_down = on_press_down.map(OnPress::Direct);
        self
    }

    /// Sets whether the [`Button`] is in a selected/toggled state.
    ///
    /// This is useful for toggle buttons or tab-like usage.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets whether the contents of the [`Button`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Button`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Button`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_pressed: bool,
    is_focused: bool,
    last_is_focused: bool,
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }

    fn focus_tier(&self) -> operation::FocusTier {
        operation::FocusTier::Control
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::core::Renderer,
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Ensure tree has children (may not be initialized in some overlay scenarios)
        if tree.children.is_empty() {
            log::warn!("Button: tree.children is empty in layout(), reinitializing");
            tree.children = self.children();
        }

        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());

        #[cfg(feature = "accessibility")]
        if let Some(info) =
            <Self as Widget<Message, Theme, Renderer>>::accessibility(self, tree, layout)
        {
            operation.accessibility(self.id.as_ref(), layout.bounds(), info);
        }

        let state = tree.state.downcast_mut::<State>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);

        if tree.children.is_empty() {
            log::warn!("Button: tree.children is empty in operate(), skipping child operation");
            return;
        }

        let Some(content_layout) = layout.children().next() else {
            log::warn!("Button: missing child layout in operate(), skipping");
            return;
        };

        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                content_layout,
                renderer,
                operation,
            );
        });
        operation.leave_container();
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if tree.children.is_empty() {
            log::warn!("Button: tree.children is empty in update(), reinitializing");
            tree.children = self.children();
        }

        let Some(content_layout) = layout.children().next() else {
            log::warn!("Button: missing child layout in update(), skipping");
            return;
        };

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            })
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let is_enabled = self.on_press.is_some() || self.on_press_down.is_some();
                if is_enabled {
                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        let state = tree.state.downcast_mut::<State>();

                        state.is_pressed = true;

                        // Fire on_press_down immediately
                        if let Some(on_press_down) = &self.on_press_down {
                            shell.publish(on_press_down.get());
                        }

                        shell.capture_event();
                    } else {
                        // Unfocus when clicked outside
                        let state = tree.state.downcast_mut::<State>();
                        if state.is_focused {
                            state.is_focused = false;
                            shell.request_redraw();
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased {
                button: mouse::Button::Left,
                ..
            })
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = &self.on_press {
                    let state = tree.state.downcast_mut::<State>();

                    if state.is_pressed {
                        state.is_pressed = false;

                        let bounds = layout.bounds();

                        if cursor.is_over(bounds) {
                            shell.publish(on_press.get());
                        }

                        shell.capture_event();
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                let state = tree.state.downcast_mut::<State>();

                state.is_pressed = false;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::Space | key::Named::Enter),
                ..
            }) => {
                if let Some(on_press) = &self.on_press {
                    let state = tree.state.downcast_ref::<State>();

                    if state.is_focused {
                        shell.publish(on_press.get());
                        shell.capture_event();
                    }
                }
            }
            #[cfg(feature = "accessibility")]
            Event::Accessibility(accessibility_event) => {
                // If widget has an explicit ID, check if the event target matches
                if let Some(id) = self.id.as_ref() {
                    if accessibility_event.target
                        != crate::core::accessibility::node_id_from_widget_id(id)
                    {
                        return;
                    }
                } else {
                    // Widget has no explicit ID - only respond if we're focused
                    // (the focus operation ensures only the correct widget is focused)
                    let state = tree.state.downcast_ref::<State>();
                    if !state.is_focused {
                        return;
                    }
                }

                // Handle screen reader "click" action
                if accessibility_event.is_click() {
                    if let Some(on_press) = &self.on_press {
                        shell.publish(on_press.get());
                        shell.capture_event();
                    }
                }
                // Handle screen reader "focus" action
                if accessibility_event.is_focus() {
                    let state = tree.state.downcast_mut::<State>();
                    state.is_focused = true;
                    shell.request_redraw();
                    shell.capture_event();
                }
                // Handle screen reader "blur" action
                if accessibility_event.is_blur() {
                    let state = tree.state.downcast_mut::<State>();
                    state.is_focused = false;
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        }

        let is_enabled = self.on_press.is_some() || self.on_press_down.is_some();
        let current_status = if !is_enabled {
            Status::Disabled
        } else if self.selected {
            Status::Selected
        } else if cursor.is_over(layout.bounds()) {
            let state = tree.state.downcast_ref::<State>();

            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            Status::Active
        };

        let state = tree.state.downcast_ref::<State>();
        if let Event::Window(crate::core::window::Event::RedrawRequested(_)) = event {
            let state = tree.state.downcast_mut::<State>();
            self.status = Some(current_status);
            state.last_is_focused = state.is_focused;
        } else if self.status.is_some_and(|status| status != current_status)
            || state.last_is_focused != state.is_focused
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let Some(content_layout) = layout.children().next() else {
            // Can happen transiently if the widget tree is out-of-sync;
            // avoid panicking during draw.
            log::warn!("Button: missing child layout in draw(), skipping");
            return;
        };

        // Compute status dynamically during draw.
        // This avoids stale hover/pressed visuals when the widget instance is reused.
        let is_enabled = self.on_press.is_some() || self.on_press_down.is_some();
        let status = if !is_enabled {
            Status::Disabled
        } else if self.selected {
            Status::Selected
        } else if cursor.is_over(bounds) {
            let state = tree.state.downcast_ref::<State>();
            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            Status::Active
        };

        let style = theme.style(&self.class, status);

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: style.snap,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        // Draw outline (for focus indication, outside the border)
        if style.outline_width > 0.0 {
            let outline_bounds = Rectangle {
                x: bounds.x - style.outline_width,
                y: bounds.y - style.outline_width,
                width: bounds.width + 2.0 * style.outline_width,
                height: bounds.height + 2.0 * style.outline_width,
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: outline_bounds,
                    border: Border {
                        color: style.outline_color,
                        width: style.outline_width,
                        radius: style.border.radius,
                    },
                    shadow: Shadow::default(),
                    snap: style.snap,
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        // Draw focus ring when focused (fallback if no outline specified)
        let state = tree.state.downcast_ref::<State>();
        if state.is_focused && is_enabled && style.outline_width == 0.0 {
            FocusRing::default().draw(renderer, bounds);
        }

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        if tree.children.is_empty() {
            log::warn!("Button: tree.children is empty in draw(), skipping content draw");
            return;
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            &viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());
        let is_enabled = self.on_press.is_some() || self.on_press_down.is_some();

        if is_mouse_over && is_enabled {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if tree.children.is_empty() {
            log::warn!("Button: tree.children is empty in overlay(), skipping");
            return None;
        }

        let Some(child_layout) = layout.children().next() else {
            log::warn!("Button: missing child layout in overlay(), skipping");
            return None;
        };

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            child_layout,
            renderer,
            viewport,
            translation,
        )
    }

    #[cfg(feature = "accessibility")]
    fn accessibility(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
    ) -> Option<crate::core::accessibility::WidgetInfo> {
        // Get label from child widget (e.g., Text) or use empty string
        let label = self
            .content
            .as_widget()
            .accessibility_label()
            .map(|s| s.into_owned())
            .unwrap_or_default();
        Some(
            crate::core::accessibility::WidgetInfo::button(label)
                .with_bounds(layout.bounds())
                .with_enabled(self.on_press.is_some()),
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}

/// The default [`Padding`] of a [`Button`].
pub const DEFAULT_PADDING: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

/// The possible status of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Button`] can be pressed.
    Active,
    /// The [`Button`] can be pressed and it is being hovered.
    Hovered,
    /// The [`Button`] is being pressed.
    Pressed,
    /// The [`Button`] is in a selected/toggled state.
    Selected,
    /// The [`Button`] cannot be pressed.
    Disabled,
}

/// The style of a button.
///
/// If not specified with [`Button::style`]
/// the theme will provide the style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the button.
    pub background: Option<Background>,
    /// The text [`Color`] of the button.
    pub text_color: Color,
    /// The [`Border`] of the button.
    pub border: Border,
    /// The [`Shadow`] of the button.
    pub shadow: Shadow,
    /// The width of the outline around the button (for focus indication).
    pub outline_width: f32,
    /// The color of the outline.
    pub outline_color: Color,
    /// The icon [`Color`] of the button (optional, for icon buttons).
    pub icon_color: Option<Color>,
    /// Whether the button should be snapped to the pixel grid.
    pub snap: bool,
}

impl Style {
    /// Updates the [`Style`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }

    /// Updates the [`Style`] with the given outline.
    pub fn with_outline(self, width: f32, color: Color) -> Self {
        Self {
            outline_width: width,
            outline_color: color,
            ..self
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            text_color: Color::BLACK,
            border: Border::default(),
            shadow: Shadow::default(),
            outline_width: 0.0,
            outline_color: Color::TRANSPARENT,
            icon_color: None,
            snap: renderer::CRISP,
        }
    }
}

/// The theme catalog of a [`Button`].
///
/// All themes that can be used with [`Button`]
/// must implement this trait.
///
/// # Example
/// ```no_run
/// # use icy_ui_widget::core::{Color, Background};
/// # use icy_ui_widget::button::{Catalog, Status, Style};
/// # struct MyTheme;
/// #[derive(Debug, Default)]
/// pub enum ButtonClass {
///     #[default]
///     Primary,
///     Secondary,
///     Danger
/// }
///
/// impl Catalog for MyTheme {
///     type Class<'a> = ButtonClass;
///     
///     fn default<'a>() -> Self::Class<'a> {
///         ButtonClass::default()
///     }
///     
///
///     fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
///         let mut style = Style::default();
///
///         match class {
///             ButtonClass::Primary => {
///                 style.background = Some(Background::Color(Color::from_rgb(0.529, 0.808, 0.921)));
///             },
///             ButtonClass::Secondary => {
///                 style.background = Some(Background::Color(Color::WHITE));
///             },
///             ButtonClass::Danger => {
///                 style.background = Some(Background::Color(Color::from_rgb(0.941, 0.502, 0.502)));
///             },
///         }
///
///         style
///     }
/// }
/// ```
///
/// Although, in order to use [`Button::style`]
/// with `MyTheme`, [`Catalog::Class`] must implement
/// `From<StyleFn<'_, MyTheme>>`.
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Button`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// A primary button; denoting a main action.
pub fn primary(theme: &Theme, status: Status) -> Style {
    let component = &theme.accent_button;
    let base = styled_component(component, theme);

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A secondary button; denoting a complementary action.
pub fn secondary(theme: &Theme, status: Status) -> Style {
    let component = &theme.button;
    let base = styled_component(component, theme);

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A success button; denoting a good outcome.
pub fn success(theme: &Theme, status: Status) -> Style {
    let component = &theme.success_button;
    let base = styled_component(component, theme);

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A warning button; denoting a risky action.
pub fn warning(theme: &Theme, status: Status) -> Style {
    let component = &theme.warning_button;
    let base = styled_component(component, theme);

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A danger button; denoting a destructive action.
pub fn danger(theme: &Theme, status: Status) -> Style {
    let component = &theme.destructive_button;
    let base = styled_component(component, theme);

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A text button; useful for links.
pub fn text_style(theme: &Theme, status: Status) -> Style {
    let component = &theme.text_button;

    let base = Style {
        text_color: component.on,
        ..Style::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            text_color: component.on.scale_alpha(0.8),
            ..base
        },
        Status::Pressed => Style {
            text_color: component.on.scale_alpha(0.6),
            ..base
        },
        Status::Selected => Style {
            text_color: theme.accent.base,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A link button; styled like a hyperlink.
pub fn link(theme: &Theme, status: Status) -> Style {
    let accent = &theme.accent;

    let base = Style {
        text_color: accent.base,
        ..Style::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            text_color: accent.hover,
            ..base
        },
        Status::Pressed => Style {
            text_color: accent.pressed,
            ..base
        },
        Status::Selected => Style {
            text_color: accent.selected,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// An icon button; minimal styling.
pub fn icon(theme: &Theme, status: Status) -> Style {
    let component = &theme.icon_button;
    let base = Style {
        background: None,
        text_color: component.on,
        icon_color: Some(component.on),
        border: border::rounded(theme.corner_radii.radius_s),
        ..Style::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            icon_color: Some(component.selected_text),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A button using background shades.
pub fn background(theme: &Theme, status: Status) -> Style {
    let component = &theme.button;
    let base = styled_component(component, theme);

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A subtle button using weak background shades.
pub fn subtle(theme: &Theme, status: Status) -> Style {
    let component = &theme.icon_button;
    let base = Style {
        background: None,
        text_color: component.on,
        border: border::rounded(theme.corner_radii.radius_s),
        ..Style::default()
    };

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            background: Some(Background::Color(component.hover)),
            ..base
        },
        Status::Pressed => Style {
            background: Some(Background::Color(component.pressed)),
            ..base
        },
        Status::Selected => Style {
            background: Some(Background::Color(component.selected)),
            text_color: component.selected_text,
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn styled_component(component: &theme::Component, theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(component.base)),
        text_color: component.on,
        border: border::rounded(theme.corner_radii.radius_s),
        ..Style::default()
    }
}

fn disabled(style: Style) -> Style {
    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        icon_color: style.icon_color.map(|c| c.scale_alpha(0.5)),
        ..style
    }
}
