# Scrollbars & Virtual Scrolling

This repository contains two related pieces of functionality:

- **Egui-style scrollbars** for `scrollable` (and `virtual_scrollable`) via `ScrollStyle`.
- **Virtual scrolling** for efficiently rendering very large content via `virtual_scrollable`.

The goal is to support thin/floating scrollbars with hover animations (similar to egui), while also enabling scalable lists/canvases where you only render what is visible.

---

## Scrollbars: `scrollable::ScrollStyle`

### What it is
`scrollable::ScrollStyle` controls the appearance and behavior of the scrollbars.

In particular it supports:

- **Floating scrollbars** (overlay the content instead of reserving layout space)
- **Thin vs solid bars** (different widths)
- **Animated visibility** (fade in/out driven by hover)

### Presets
There are three built-in presets:

- `scrollable::ScrollStyle::floating()`
- `scrollable::ScrollStyle::thin()`
- `scrollable::ScrollStyle::solid()`

Use them directly in your style function (see below), or treat them as a baseline and override fields.

### Hover animation model
The scroll widgets track a `hover_factor: f32` (typically in $[0, 1]$), which is animated when the mouse enters/leaves the scroll area.

`ScrollStyle` exposes helpers like:

- `handle_opacity(hover_factor, is_interacting)`
- `background_opacity(hover_factor, is_interacting)`

so a style function can compute final colors/alpha values consistently.

### Styling a scrollable
You customize scrollbars by providing a style function:

```rust
use iced::widget::scrollable;
use iced::{Color, Theme};

fn style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let base = scrollable::default(theme, status);

    let scroll = scrollable::ScrollStyle::floating();

    scrollable::Style {
        scroll,
        ..base
    }
}
```

You can also react to `status` (hovered/dragged) and `hover_factor` to blend colors.

---

## Kinetic Scrolling & Smooth Scroll-To

Both `scrollable` and `virtual_scrollable` support **kinetic scrolling** (momentum/flick scrolling) and **smooth animated scroll-to**.

### Kinetic Scrolling (Touch/Flick)
When the user scrolls via touch and releases their finger, the content continues scrolling with momentum that naturally decelerates:

- **Velocity tracking**: Touch velocity is tracked with exponential smoothing during finger movement
- **Momentum physics**: On finger release, scrolling continues with physics-based momentum
- **Friction decay**: Exponential friction (factor 5.0) provides natural deceleration
- **Auto-stop**: Scrolling stops when velocity drops below 1.0 px/s or hits content edges
- **Interruption**: Wheel scrolling or new touch interactions immediately cancel kinetic motion

This behavior is automatic and requires no configuration.

### Smooth Scroll-To Animation
For programmatic scrolling, use `scroll_to_animated` instead of `scroll_to` for smooth animated transitions:

```rust
use iced::widget::scrollable;
use iced::widget::scrollable::AbsoluteOffset;

// Immediate scroll (jumps instantly)
scrollable::scroll_to(id.clone(), AbsoluteOffset { x: 0.0, y: 500.0 })

// Animated scroll (smooth transition with ease-out cubic easing)
scrollable::scroll_to_animated(id.clone(), AbsoluteOffset { x: 0.0, y: 500.0 })
```

The animation uses ease-out cubic easing ($1 - (1 - t)^3$) for a smooth, natural feel.

You can also call `scroll_to_animated()` directly on the widget's `State`:

```rust
// In your update logic with access to state
state.scroll_to_animated(AbsoluteOffset { x: 0.0, y: 500.0 });
```

### State Query Methods
The `State` struct provides methods to check animation status:

- `is_kinetic_active()` – Returns `true` if kinetic scrolling is in progress
- `is_scroll_to_animating(now)` – Returns `true` if a scroll-to animation is active

---

## Virtual scrolling: `virtual_scrollable`

### When to use it
Use `virtual_scrollable` when your content is extremely large:

- lists with tens/hundreds of thousands of rows
- huge 2D canvases
- grids where only a small portion is visible at any time

It avoids building widgets for the entire content. Instead, it calls your view callback only for the **currently visible viewport**.

### Key idea: viewport-driven rendering (no internal content translation)
`virtual_scrollable` is designed to be **purely virtual**:

- The widget maintains a scroll offset.
- It computes the **visible rectangle in content coordinates**.
- It calls your callback with that rectangle.
- **You render the content for that rectangle.**

Importantly: your content is not expected to be a giant widget that gets moved/translated. The scroll position is expressed through the viewport coordinates you receive.

### API overview
There are two primary entry points:

#### 1) Uniform rows: `show_rows`
For row-based lists with a constant row height:

```rust
use iced::widget::{column, container, text, virtual_scrollable};
use iced::{Element, Length};

const TOTAL_ROWS: usize = 100_000;

fn view<'a, Message>() -> Element<'a, Message> {
    let row_height = 30.0;

    virtual_scrollable::show_rows(row_height, TOTAL_ROWS, move |visible| {
        column(visible.map(|i| {
            container(text(format!("Row {}", i + 1)))
                .height(row_height)
                .width(Length::Fill)
                .into()
        }))
        .into()
    })
    .into()
}
```

Your callback receives `Range<usize>` of visible rows.

#### 2) Custom virtualization: `show_viewport`
For arbitrary content (2D canvases, sparse grids, variable-sized items):

```rust
use iced::widget::{column, text, virtual_scrollable};
use iced::{Element, Rectangle, Size};

fn view<'a, Message>() -> Element<'a, Message> {
    let content_size = Size::new(100_000.0, 100_000.0);

    virtual_scrollable::show_viewport(content_size, move |viewport: Rectangle| {
        // viewport.x / viewport.y are the top-left coordinates in content space.
        // viewport.width / viewport.height are the visible size.
        column![
            text(format!(
                "visible rect: x={:.0} y={:.0} w={:.0} h={:.0}",
                viewport.x, viewport.y, viewport.width, viewport.height
            )),
        ]
        .into()
    })
    .into()
}
```

Your callback receives a `Rectangle` representing the visible area in content coordinates.

### Scroll events
You can observe scrolling using `.on_scroll(...)`.

The callback receives `virtual_scrollable::Viewport`, which can provide:

- `absolute_offset()`
- `relative_offset()`
- `visible_rect()`

`relative_offset()` is clamped to avoid `NaN` when content does not overflow.

---

## Using the same scrollbar styling for `scrollable` and `virtual_scrollable`

Both widgets accept `.style(|theme, status| ...)` and use the same `scrollable::Status` model (including `hover_factor`).

That means you can share a single scrollbar style function across normal and virtual scrolling.

---

## Tips

- Prefer `show_rows` when you can: it’s simpler and avoids manual math.
- In `show_viewport`, always compute visible indices/tiles from `viewport.x/y/width/height`.
- Keep the number of widgets you produce per frame bounded (e.g. only visible tiles + a small buffer).

---

## Related docs

- [doc/MOUSE_EVENTS.md](doc/MOUSE_EVENTS.md)
