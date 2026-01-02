//! iced is a cross-platform GUI library focused on simplicity and type-safety.
//! Inspired by [Elm].
//!
//! [Elm]: https://elm-lang.org/
//!
//! # Disclaimer
//! iced is __experimental__ software. If you expect the documentation to hold your hand
//! as you learn the ropes, you are in for a frustrating experience.
//!
//! The library leverages Rust to its full extent: ownership, borrowing, lifetimes, futures,
//! streams, first-class functions, trait bounds, closures, and more. This documentation
//! is not meant to teach you any of these. Far from it, it will assume you have __mastered__
//! all of them.
//!
//! Furthermore—just like Rust—iced is very unforgiving. It will not let you easily cut corners.
//! The type signatures alone can be used to learn how to use most of the library.
//! Everything is connected.
//!
//! Therefore, iced is easy to learn for __advanced__ Rust programmers; but plenty of patient
//! beginners have learned it and had a good time with it. Since it leverages a lot of what
//! Rust has to offer in a type-safe way, it can be a great way to discover Rust itself.
//!
//! If you don't like the sound of that, you expect to be spoonfed, or you feel frustrated
//! and struggle to use the library; then I recommend you to wait patiently until [the book]
//! is finished.
//!
//! [the book]: https://book.iced.rs
//!
//! # The Pocket Guide
//! Start by calling [`run`]:
//!
//! ```no_run,standalone_crate
//! pub fn main() -> iced::Result {
//!     iced::run(update, view)
//! }
//! # fn update(state: &mut (), message: ()) {}
//! # fn view(state: &()) -> icy_ui::Element<'_, ()> { iced::widget::text("").into() }
//! ```
//!
//! Define an `update` function to __change__ your state:
//!
//! ```standalone_crate
//! fn update(counter: &mut u64, message: Message) {
//!     match message {
//!         Message::Increment => *counter += 1,
//!     }
//! }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! ```
//!
//! Define a `view` function to __display__ your state:
//!
//! ```standalone_crate
//! use icy_ui::widget::{button, text};
//! use icy_ui::Element;
//!
//! fn view(counter: &u64) -> Element<'_, Message> {
//!     button(text(counter)).on_press(Message::Increment).into()
//! }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! ```
//!
//! And create a `Message` enum to __connect__ `view` and `update` together:
//!
//! ```standalone_crate
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Increment,
//! }
//! ```
//!
//! ## Custom State
//! You can define your own struct for your state:
//!
//! ```standalone_crate
//! #[derive(Default)]
//! struct Counter {
//!     value: u64,
//! }
//! ```
//!
//! But you have to change `update` and `view` accordingly:
//!
//! ```standalone_crate
//! # struct Counter { value: u64 }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! # use icy_ui::widget::{button, text};
//! # use icy_ui::Element;
//! fn update(counter: &mut Counter, message: Message) {
//!     match message {
//!         Message::Increment => counter.value += 1,
//!     }
//! }
//!
//! fn view(counter: &Counter) -> Element<'_, Message> {
//!     button(text(counter.value)).on_press(Message::Increment).into()
//! }
//! ```
//!
//! ## Widgets and Elements
//! The `view` function must return an [`Element`]. An [`Element`] is just a generic [`widget`].
//!
//! The [`widget`] module contains a bunch of functions to help you build
//! and use widgets.
//!
//! Widgets are configured using the builder pattern:
//!
//! ```standalone_crate
//! # struct Counter { value: u64 }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! use icy_ui::widget::{button, column, text};
//! use icy_ui::Element;
//!
//! fn view(counter: &Counter) -> Element<'_, Message> {
//!     column![
//!         text(counter.value).size(20),
//!         button("Increment").on_press(Message::Increment),
//!     ]
//!     .spacing(10)
//!     .into()
//! }
//! ```
//!
//! A widget can be turned into an [`Element`] by calling `into`.
//!
//! Widgets and elements are generic over the message type they produce. The
//! [`Element`] returned by `view` must have the same `Message` type as
//! your `update`.
//!
//! ## Layout
//! There is no unified layout system in iced. Instead, each widget implements
//! its own layout strategy.
//!
//! Building your layout will often consist in using a combination of
//! [rows], [columns], and [containers]:
//!
//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use icy_ui::widget::{column, container, row};
//! use icy_ui::{Fill, Element};
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     container(
//!         column![
//!             "Top",
//!             row!["Left", "Right"].spacing(10),
//!             "Bottom"
//!         ]
//!         .spacing(10)
//!     )
//!     .padding(10)
//!     .center_x(Fill)
//!     .center_y(Fill)
//!     .into()
//! }
//! ```
//!
//! Rows and columns lay out their children horizontally and vertically,
//! respectively. [Spacing] can be easily added between elements.
//!
//! Containers position or align a single widget inside their bounds.
//!
//! [rows]: widget::Row
//! [columns]: widget::Column
//! [containers]: widget::Container
//! [Spacing]: widget::Column::spacing
//!
//! ## Sizing
//! The width and height of widgets can generally be defined using a [`Length`].
//!
//! - [`Fill`] will make the widget take all the available space in a given axis.
//! - [`Shrink`] will make the widget use its intrinsic size.
//!
//! Most widgets use a [`Shrink`] sizing strategy by default, but will inherit
//! a [`Fill`] strategy from their children.
//!
//! A fixed numeric [`Length`] in [`Pixels`] can also be used:
//!
//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use icy_ui::widget::container;
//! use icy_ui::Element;
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     container("I am 300px tall!").height(300).into()
//! }
//! ```
//!
//! ## Theming
//! The default [`Theme`] of an application can be changed by defining a `theme`
//! function and leveraging the [`Application`] builder, instead of directly
//! calling [`run`]:
//!
//! ```no_run,standalone_crate
//! # struct State;
//! use icy_ui::Theme;
//!
//! pub fn main() -> iced::Result {
//!     iced::application(new, update, view)
//!         .theme(theme)
//!         .run()
//! }
//!
//! fn new() -> State {
//!     // ...
//!     # State
//! }
//!
//! fn theme(state: &State) -> Theme {
//!     Theme::TokyoNight
//! }
//! # fn update(state: &mut State, message: ()) {}
//! # fn view(state: &State) -> icy_ui::Element<'_, ()> { iced::widget::text("").into() }
//! ```
//!
//! The `theme` function takes the current state of the application, allowing the
//! returned [`Theme`] to be completely dynamic—just like `view`.
//!
//! There are a bunch of built-in [`Theme`] variants at your disposal, but you can
//! also [create your own](Theme::custom).
//!
//! ## Styling
//! As with layout, iced does not have a unified styling system. However, all
//! of the built-in widgets follow the same styling approach.
//!
//! The appearance of a widget can be changed by calling its `style` method:
//!
//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use icy_ui::widget::container;
//! use icy_ui::Element;
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     container("I am a rounded box!").style(container::rounded_box).into()
//! }
//! ```
//!
//! The `style` method of a widget takes a closure that, given the current active
//! [`Theme`], returns the widget style:
//!
//! ```standalone_crate
//! # struct State;
//! # #[derive(Clone)]
//! # enum Message {}
//! use icy_ui::widget::button;
//! use icy_ui::{Element, Theme};
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     button("I am a styled button!").style(|theme: &Theme, status| {
//!         let palette = theme.extended_palette();
//!
//!         match status {
//!             button::Status::Active => {
//!                 button::Style::default()
//!                    .with_background(palette.success.strong.color)
//!             }
//!             _ => button::primary(theme, status),
//!         }
//!     })
//!     .into()
//! }
//! ```
//!
//! Widgets that can be in multiple different states will also provide the closure
//! with some [`Status`], allowing you to use a different style for each state.
//!
//! You can extract the [`Palette`] colors of a [`Theme`] with the [`palette`] or
//! [`extended_palette`] methods.
//!
//! Most widgets provide styling functions for your convenience in their respective modules;
//! like [`container::rounded_box`], [`button::primary`], or [`text::danger`].
//!
//! [`Status`]: widget::button::Status
//! [`palette`]: Theme::palette
//! [`extended_palette`]: Theme::extended_palette
//! [`container::rounded_box`]: widget::container::rounded_box
//! [`button::primary`]: widget::button::primary
//! [`text::danger`]: widget::text::danger
//!
//! ## Concurrent Tasks
//! The `update` function can _optionally_ return a [`Task`].
//!
//! A [`Task`] can be leveraged to perform asynchronous work, like running a
//! future or a stream:
//!
//! ```standalone_crate
//! # #[derive(Clone)]
//! # struct Weather;
//! use icy_ui::Task;
//!
//! struct State {
//!     weather: Option<Weather>,
//! }
//!
//! enum Message {
//!    FetchWeather,
//!    WeatherFetched(Weather),
//! }
//!
//! fn update(state: &mut State, message: Message) -> Task<Message> {
//!     match message {
//!         Message::FetchWeather => Task::perform(
//!             fetch_weather(),
//!             Message::WeatherFetched,
//!         ),
//!         Message::WeatherFetched(weather) => {
//!             state.weather = Some(weather);
//!
//!             Task::none()
//!        }
//!     }
//! }
//!
//! async fn fetch_weather() -> Weather {
//!     // ...
//!     # unimplemented!()
//! }
//! ```
//!
//! Tasks can also be used to interact with the iced runtime. Some modules
//! expose functions that create tasks for different purposes—like [changing
//! window settings](window#functions), [focusing a widget](widget::operation::focus_next), or
//! [querying its visible bounds](widget::selector::find).
//!
//! Like futures and streams, tasks expose [a monadic interface](Task::then)—but they can also be
//! [mapped](Task::map), [chained](Task::chain), [batched](Task::batch), [canceled](Task::abortable),
//! and more.
//!
//! ## Passive Subscriptions
//! Applications can subscribe to passive sources of data—like time ticks or runtime events.
//!
//! You will need to define a `subscription` function and use the [`Application`] builder:
//!
//! ```no_run,standalone_crate
//! # struct State;
//! use icy_ui::window;
//! use icy_ui::{Size, Subscription};
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     WindowResized(Size),
//! }
//!
//! pub fn main() -> iced::Result {
//!     iced::application(new, update, view)
//!         .subscription(subscription)
//!         .run()
//! }
//!
//! fn subscription(state: &State) -> Subscription<Message> {
//!     window::resize_events().map(|(_id, size)| Message::WindowResized(size))
//! }
//! # fn new() -> State { State }
//! # fn update(state: &mut State, message: Message) {}
//! # fn view(state: &State) -> icy_ui::Element<'_, Message> { iced::widget::text("").into() }
//! ```
//!
//! A [`Subscription`] is [a _declarative_ builder of streams](Subscription#the-lifetime-of-a-subscription)
//! that are not allowed to end on their own. Only the `subscription` function
//! dictates the active subscriptions—just like `view` fully dictates the
//! visible widgets of your user interface, at every moment.
//!
//! As with tasks, some modules expose convenient functions that build a [`Subscription`] for you—like
//! [`time::every`] which can be used to listen to time, or [`keyboard::listen`] which will notify you
//! of any keyboard events. But you can also create your own with [`Subscription::run`] and [`run_with`].
//!
//! [`run_with`]: Subscription::run_with
//!
//! ## Scaling Applications
//! The `update`, `view`, and `Message` triplet composes very nicely.
//!
//! A common pattern is to leverage this composability to split an
//! application into different screens:
//!
//! ```standalone_crate
//! # mod contacts {
//! #     use icy_ui::{Element, Task};
//! #     pub struct Contacts;
//! #     impl Contacts {
//! #         pub fn update(&mut self, message: Message) -> Action { unimplemented!() }
//! #         pub fn view(&self) -> Element<Message> { unimplemented!() }
//! #     }
//! #     #[derive(Debug, Clone)]
//! #     pub enum Message {}
//! #     pub enum Action { None, Run(Task<Message>), Chat(()) }
//! # }
//! # mod conversation {
//! #     use icy_ui::{Element, Task};
//! #     pub struct Conversation;
//! #     impl Conversation {
//! #         pub fn new(contact: ()) -> (Self, Task<Message>) { unimplemented!() }
//! #         pub fn update(&mut self, message: Message) -> Task<Message> { unimplemented!() }
//! #         pub fn view(&self) -> Element<Message> { unimplemented!() }
//! #     }
//! #     #[derive(Debug, Clone)]
//! #     pub enum Message {}
//! # }
//! use contacts::Contacts;
//! use conversation::Conversation;
//!
//! use icy_ui::{Element, Task};
//!
//! struct State {
//!     screen: Screen,
//! }
//!
//! enum Screen {
//!     Contacts(Contacts),
//!     Conversation(Conversation),
//! }
//!
//! enum Message {
//!    Contacts(contacts::Message),
//!    Conversation(conversation::Message)
//! }
//!
//! fn update(state: &mut State, message: Message) -> Task<Message> {
//!     match message {
//!         Message::Contacts(message) => {
//!             if let Screen::Contacts(contacts) = &mut state.screen {
//!                 let action = contacts.update(message);
//!
//!                 match action {
//!                     contacts::Action::None => Task::none(),
//!                     contacts::Action::Run(task) => task.map(Message::Contacts),
//!                     contacts::Action::Chat(contact) => {
//!                         let (conversation, task) = Conversation::new(contact);
//!
//!                         state.screen = Screen::Conversation(conversation);
//!
//!                         task.map(Message::Conversation)
//!                     }
//!                  }
//!             } else {
//!                 Task::none()    
//!             }
//!         }
//!         Message::Conversation(message) => {
//!             if let Screen::Conversation(conversation) = &mut state.screen {
//!                 conversation.update(message).map(Message::Conversation)
//!             } else {
//!                 Task::none()    
//!             }
//!         }
//!     }
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     match &state.screen {
//!         Screen::Contacts(contacts) => contacts.view().map(Message::Contacts),
//!         Screen::Conversation(conversation) => conversation.view().map(Message::Conversation),
//!     }
//! }
//! ```
//!
//! The `update` method of a screen can return an `Action` enum that can be leveraged by the parent to
//! execute a task or transition to a completely different screen altogether. The variants of `Action` can
//! have associated data. For instance, in the example above, the `Conversation` screen is created when
//! `Contacts::update` returns an `Action::Chat` with the selected contact.
//!
//! Effectively, this approach lets you "tell a story" to connect different screens together in a type safe
//! way.
//!
//! Furthermore, functor methods like [`Task::map`], [`Element::map`], and [`Subscription::map`] make composition
//! seamless.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/bdf0430880f5c29443f5f0a0ae4895866dfef4c6/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
use icy_ui_widget::graphics;
use icy_ui_widget::renderer;
use icy_ui_winit as shell;
use icy_ui_winit::core;
use icy_ui_winit::program;
use icy_ui_winit::runtime;

pub use icy_ui_futures::futures;
pub use icy_ui_futures::stream;

#[cfg(not(any(
    target_arch = "wasm32",
    feature = "thread-pool",
    feature = "tokio",
    feature = "smol"
)))]
compile_error!(
    "No futures executor has been enabled! You must enable an \
    executor feature.\n\
    Available options: thread-pool, tokio, or smol."
);

#[cfg(all(
    target_family = "unix",
    not(target_os = "macos"),
    not(feature = "wayland"),
    not(feature = "x11"),
))]
compile_error!(
    "No Unix display server backend has been enabled. You must enable a \
    display server feature.\n\
    Available options: x11, wayland."
);

#[cfg(feature = "highlighter")]
pub use icy_ui_highlighter as highlighter;

#[cfg(feature = "wgpu")]
pub use icy_ui_renderer::wgpu::wgpu;

mod error;

pub mod application;
pub mod daemon;
pub mod time;
pub mod window;

#[cfg(feature = "advanced")]
pub mod advanced;

pub use crate::core::alignment;
pub use crate::core::animation;
pub use crate::core::border;
pub use crate::core::color;
pub use crate::core::gradient;
pub use crate::core::padding;
pub use crate::core::theme;
pub use crate::core::{
    Alignment, Animation, Background, Border, Color, ContentFit, Degrees, Function, Gradient,
    Length, Never, Padding, Pixels, Point, Radians, Rectangle, Rotation, Settings, Shadow, Size,
    Theme, Transformation, Vector, never,
};
pub use crate::program::Preset;
pub use crate::program::message;
pub use crate::runtime::exit;
pub use icy_ui_futures::Subscription;

pub use Alignment::Center;
pub use Length::{Fill, FillPortion, Shrink};
pub use alignment::Horizontal::{Left, Right};
pub use alignment::Vertical::{Bottom, Top};

pub mod debug {
    //! Debug your applications.
    pub use icy_ui_debug::{Span, time, time_with};
}

pub mod task {
    //! Create runtime tasks.
    pub use crate::runtime::task::{Handle, Task};

    #[cfg(feature = "sipper")]
    pub use crate::runtime::task::{Never, Sipper, Straw, sipper, stream};
}

pub mod clipboard {
    //! Access the clipboard.
    //!
    //! # Example
    //!
    //! ```no_run
    //! use icy_ui::clipboard::{STANDARD, PRIMARY, Format};
    //!
    //! // Read text from the standard clipboard
    //! let task = STANDARD.read_text();
    //!
    //! // Write text to the primary clipboard
    //! let task = PRIMARY.write_text("Hello".to_string());
    //!
    //! // Use format constants
    //! let task = STANDARD.read_format(Format::Image.formats());
    //!
    //! // Use the builder pattern
    //! let task = STANDARD.write()
    //!     .html("<b>Hello</b>".to_string())
    //!     .text("Hello".to_string())
    //!     .finish();
    //! ```

    pub use crate::runtime::clipboard::{Error, Format, PRIMARY, STANDARD, Target, WriteBuilder};
}

pub mod dnd {
    //! Drag and drop support.
    //!
    //! This module provides functionality for initiating and handling drag and drop
    //! operations, supporting both internal (widget-to-widget) and external
    //! (cross-application) drag and drop.
    //!
    //! # Example
    //! ```ignore
    //! use icy_ui::dnd::{self, DragData};
    //!
    //! // Start a drag with text data
    //! let task = dnd::start_drag(
    //!     DragData::from_text("Hello, world!"),
    //!     None,
    //! );
    //! ```
    pub use crate::core::dnd::{
        DndAction, DragData, DragIcon, DragSourceEvent, DropResult, DropTargetEvent, DropZone,
    };
    pub use crate::runtime::dnd::{
        accept_drag, reject_drag, request_data, set_drop_zones, start_drag, start_drag_with_actions,
    };
}

#[cfg(feature = "accessibility")]
pub mod accessibility {
    //! Accessibility support for screen readers and assistive technologies.
    //!
    //! This module provides types for building an accessibility tree that can be
    //! consumed by screen readers like NVDA (Windows), VoiceOver (macOS), and
    //! Orca (Linux).
    //!
    //! # Overview
    //!
    //! Accessibility support is built on [AccessKit](https://accesskit.dev/), which
    //! provides cross-platform accessibility via native APIs:
    //! - **Windows**: UI Automation
    //! - **macOS**: NSAccessibility
    //! - **Linux**: AT-SPI (via D-Bus)
    //!
    //! # Events
    //!
    //! When a screen reader user interacts with your app, you'll receive
    //! [`Event::Accessibility`](crate::Event::Accessibility) events that contain
    //! action requests like "click", "focus", or "set value".
    //!
    //! # Example
    //! ```ignore
    //! use icy_ui::accessibility::{WidgetInfo, Action, Event};
    //!
    //! // Create accessibility info for a button
    //! let info = WidgetInfo::button("Press me!");
    //!
    //! // Handle accessibility events in update()
    //! match event {
    //!     Event::Accessibility(acc_event) if acc_event.is_click() => {
    //!         // Handle screen reader "click" action
    //!     }
    //!     _ => {}
    //! }
    //! ```
    pub use crate::core::accessibility::{
        AccessibilityState, Action, ActionData, ActionRequest, Event, Node, NodeId, Role, Tree,
        TreeUpdate, WidgetInfo, node_id, node_id_from_widget_id,
    };
    pub use crate::runtime::accessibility::{Priority, announce, focus};
}

pub mod executor {
    //! Choose your preferred executor to power your application.
    pub use icy_ui_futures::Executor;
    pub use icy_ui_futures::backend::default::Executor as Default;
}

pub mod font {
    //! Load and use fonts.
    pub use crate::core::font::*;
    pub use crate::runtime::font::*;
}

pub mod event {
    //! Handle events of a user interface.
    pub use crate::core::event::{Event, Status};
    pub use icy_ui_futures::event::{listen, listen_raw, listen_url, listen_with};
}

pub mod keyboard {
    //! Listen and react to keyboard events.
    pub use crate::core::keyboard::key;
    pub use crate::core::keyboard::{Event, Key, Location, Modifiers};
    pub use icy_ui_futures::keyboard::listen;
}

pub mod mouse {
    //! Listen and react to mouse events.
    pub use crate::core::mouse::{Button, Cursor, Event, Interaction, ScrollDelta};
}

pub mod system {
    //! Retrieve system information.
    pub use crate::runtime::system::{theme, theme_changes};

    #[cfg(feature = "sysinfo")]
    pub use crate::runtime::system::{Information, information};
}

pub mod menu {
    //! Application menu and context menu model types.
    //!
    //! This module provides a unified API for menus in icy_ui:
    //! - Application menu bars (native on macOS, widget-based on other platforms)
    //! - Context menus (native on macOS, widget-based overlay on other platforms)
    //!
    //! # Macros for Stable IDs
    //!
    //! Use the provided macros to create menu items with automatically stable IDs:
    //!
    //! ```ignore
    //! use icy_ui::menu;
    //!
    //! let file_menu = menu::submenu!("File", [
    //!     menu::item!("New", Message::New),
    //!     menu::item!("Open", Message::Open),
    //!     menu::separator!(),
    //!     menu::item!("Save", Message::Save),
    //! ]);
    //! ```
    //!
    //! # Keyboard Shortcuts
    //!
    //! Add keyboard shortcuts to menu items:
    //!
    //! ```ignore
    //! use icy_ui::menu::{self, MenuShortcut};
    //! use icy_ui::keyboard::Key;
    //!
    //! // Using the macro with shortcut
    //! menu::item!("Save", Message::Save, MenuShortcut::cmd(Key::Character("s".into())))
    //!
    //! // Or using the builder pattern
    //! menu::item!("Save", Message::Save)
    //!     .shortcut(MenuShortcut::cmd(Key::Character("s".into())))
    //! ```
    //!
    //! # Context Menus
    //!
    //! Use the `context_menu` widget which automatically uses native menus on macOS:
    //!
    //! ```ignore
    //! use icy_ui::widget::menu::context_menu;
    //! use icy_ui::menu;
    //!
    //! let nodes = vec![
    //!     menu::item!("Cut", Message::Cut),
    //!     menu::item!("Copy", Message::Copy),
    //! ];
    //!
    //! context_menu(my_content, &nodes)
    //! ```

    // Re-export all core menu types
    pub use icy_ui_core::menu::{
        // Types
        AppMenu,
        ContextMenuItem,
        ContextMenuItemKind,
        MenuContext,
        MenuId,
        MenuKind,
        MenuNode,
        MenuRole,
        MenuShortcut,
        WindowInfo,
        // Helper functions
        fnv1a_hash_location,
        fnv1a_hash_str,
    };

    pub use icy_ui_core::menu::{ContextMenuItem as MenuItem, ContextMenuItemKind as MenuItemKind};

    // Re-export runtime utilities
    pub use crate::runtime::context_menu::menu_nodes_to_items;

    // Re-export macros from core
    pub use icy_ui_core::{
        menu_about as about, menu_check_item as check_item, menu_item as item,
        menu_preferences as preferences, menu_quit as quit, menu_separator as separator,
        menu_submenu as submenu,
    };
}

pub mod overlay {
    //! Display interactive elements on top of other widgets.

    /// A generic overlay.
    ///
    /// This is an alias of an [`overlay::Element`] with a default `Renderer`.
    ///
    /// [`overlay::Element`]: crate::core::overlay::Element
    pub type Element<'a, Message, Theme = crate::Renderer, Renderer = crate::Renderer> =
        crate::core::overlay::Element<'a, Message, Theme, Renderer>;

    pub use icy_ui_widget::overlay::*;
}

pub mod touch {
    //! Listen and react to touch events.
    pub use crate::core::touch::{Event, Finger};
}

#[allow(hidden_glob_reexports)]
pub mod widget {
    //! Use the built-in widgets or create your own.
    pub use icy_ui_runtime::widget::*;
    pub use icy_ui_widget::*;

    #[cfg(feature = "image")]
    pub mod image {
        //! Images display raster graphics in different formats (PNG, JPG, etc.).
        pub use icy_ui_runtime::image::{Allocation, Error, allocate};
        pub use icy_ui_widget::image::*;
    }

    // We hide the re-exported modules by `icy_ui_widget`
    mod core {}
    mod graphics {}
    mod renderer {}
}

pub use application::Application;
pub use daemon::Daemon;
pub use error::Error;
pub use event::Event;
pub use executor::Executor;
pub use font::Font;
pub use program::Program;
pub use renderer::Renderer;
pub use task::Task;
pub use window::Window;

#[doc(inline)]
pub use application::application;
#[doc(inline)]
pub use daemon::daemon;

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> =
    crate::core::Element<'a, Message, Theme, Renderer>;

/// The result of running an iced program.
pub type Result = std::result::Result<(), Error>;

/// Runs a basic iced application with default [`Settings`] given its update
/// and view logic.
///
/// This is equivalent to chaining [`application()`] with [`Application::run`].
///
/// # Example
/// ```no_run,standalone_crate
/// use icy_ui::widget::{button, column, text, Column};
///
/// pub fn main() -> iced::Result {
///     iced::run(update, view)
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     Increment,
/// }
///
/// fn update(value: &mut u64, message: Message) {
///     match message {
///         Message::Increment => *value += 1,
///     }
/// }
///
/// fn view(value: &u64) -> Column<Message> {
///     column![
///         text(value),
///         button("+").on_press(Message::Increment),
///     ]
/// }
/// ```
#[cfg(all(
    feature = "debug",
    not(feature = "tester"),
    not(target_arch = "wasm32")
))]
pub fn run<State, Message, Renderer>(
    update: impl application::UpdateFn<State, Message> + 'static,
    view: impl for<'a> application::ViewFn<'a, State, Message, Theme, Renderer> + 'static,
) -> Result
where
    State: Default + 'static,
    Message: Send + message::MaybeDebug + message::MaybeClone + 'static,
    Renderer: program::Renderer + 'static,
{
    application(State::default, update, view).run()
}

/// A simple iced application.
#[cfg(not(all(
    feature = "debug",
    not(feature = "tester"),
    not(target_arch = "wasm32")
)))]
pub fn run<State, Message, Theme, Renderer>(
    update: impl application::UpdateFn<State, Message> + 'static,
    view: impl for<'a> application::ViewFn<'a, State, Message, Theme, Renderer> + 'static,
) -> Result
where
    State: Default + 'static,
    Message: Send + message::MaybeDebug + message::MaybeClone + 'static,
    Theme: theme::Base + 'static,
    Renderer: program::Renderer + 'static,
{
    application(State::default, update, view).run()
}
