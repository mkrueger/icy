# Scrollbars & Virtual Scrolling

This repository provides two related features:

- **Modern scrollbars** for `scrollable` and `virtual_scrollable` (hover-animated width/opacity, presets).
- **Virtual scrolling** (render only what is visible) via `virtual_scrollable`.

The implementation lives in `crates/widget/src/scrolling/*`.

## Scrollbar system overview

There are two layers that work together:

- **Placement & hit-testing**: the scroll directions (horizontal/vertical) and their geometry (alignment, widths, margins, spacing).
- **Appearance & hover animation**: `scrollable::ScrollStyle`.

The key idea is that the widget computes a `hover_factor` that animates between $0$ and $1$ when the pointer enters/leaves the scroll area. `ScrollStyle` uses this to derive the *current* width and opacities.

### `scrollable::Status` and `hover_factor`

Style functions receive a `scrollable::Status`, which carries the current `hover_factor`:

```rust
use iced::widget::scrollable;

fn hover_factor(status: scrollable::Status) -> f32 {
	match status {
		scrollable::Status::Active { hover_factor, .. }
		| scrollable::Status::Hovered { hover_factor, .. }
		| scrollable::Status::Dragged { hover_factor, .. } => hover_factor,
	}
}
```

### `scrollable::ScrollStyle`

`ScrollStyle` is the configuration used to compute the scrollbar visuals. It includes (among others):

- Whether the scrollbar **floats** over content (`floating`) or reserves space.
- A **base width** and a **hover-expanded width**.
- Opacity settings for the rail/handle (idle vs hovered).
- A minimum handle length.

It also provides helper methods like `current_width(hover_factor)`, `rail_opacity(hover_factor)` and `handle_opacity(hover_factor)`.

### Presets: `floating`, `thin`, `solid`

The easiest way to style scrollbars is to start from a preset:

- `scrollable::floating(theme, status)`
- `scrollable::thin(theme, status)`
- `scrollable::solid(theme, status)`

These return a complete `scrollable::Style`.

Example (tweak a preset):

```rust
use iced::widget::scrollable;
use iced::Theme;

fn style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
	let mut s = scrollable::floating(theme, status);

	// Example tweak: keep floating, but reserve a small gutter.
	s.scroll.floating_allocated_width = 2.0;
	s
}
```

## Kinetic scrolling and smooth scroll-to

Both `scrollable` and `virtual_scrollable` support:

- **Kinetic scrolling** (momentum after a touch drag).
- **Smooth scroll-to animations**.

### Programmatic scrolling

Use `scroll_to_animated` to smoothly scroll to an absolute offset:

```rust
use iced::widget::scrollable;
use iced::widget::scrollable::AbsoluteOffset;

scrollable::scroll_to_animated(
	"my_scroll".into(),
	AbsoluteOffset {
		x: Some(0.0),
		y: Some(500.0),
	},
);
```

## Virtual scrolling: `virtual_scrollable`

`virtual_scrollable` is designed for very large content. Instead of building a widget tree for all content, it asks you to render only what is visible.

There are two entry points:

### `virtual_scrollable::show_viewport`

Use this when content is not a simple list (or row heights vary). You specify a total `content_size`, and you render based on the `viewport` rectangle (in content coordinates):

```rust
use iced::widget::virtual_scrollable;
use iced::{Element, Rectangle, Size};

fn view<'a, Message>() -> Element<'a, Message> {
	virtual_scrollable::show_viewport(Size::new(2000.0, 2_000_000.0), |viewport: Rectangle| {
		// Render only what intersects `viewport`.
		// Your content should use the same coordinate system.
		todo!()
	})
	.into()
}
```

### `virtual_scrollable::show_rows`

Convenience helper for **uniform row height** lists. It returns the visible row range; you render just those rows:

```rust
use iced::widget::{column, text, virtual_scrollable};
use iced::Element;

fn view<'a, Message>() -> Element<'a, Message> {
	let row_height = 20.0;
	let total_rows = 100_000;

	virtual_scrollable::show_rows(row_height, total_rows, |range| {
		range
			.fold(column![], |col, row| col.push(text(format!("Row {row}"))))
			.into()
	})
	.into()
}
```

### Caching: `cache_key`

`virtual_scrollable` can cache the generated viewport content. If your content changes without changing scroll position/viewport size, call `.cache_key(new_key)` to invalidate the cache.

## Styling: shared between `scrollable` and `virtual_scrollable`

Both widgets accept a `.style(|theme, status| ...)` function and share the same `scrollable::Status` model (including `hover_factor`).
