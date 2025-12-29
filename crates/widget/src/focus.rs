//! Focus utilities and visual indicators.
//!
//! This module provides helpers for drawing consistent focus rings
//! across widgets.

use crate::core::renderer::{self};
use crate::core::{Border, Color, Rectangle, Theme};

/// The default width of a focus ring border.
pub const FOCUS_RING_WIDTH: f32 = 2.0;

/// The default offset of the focus ring from the widget bounds.
pub const FOCUS_RING_OFFSET: f32 = 2.0;

/// The default border radius for focus rings.
pub const FOCUS_RING_RADIUS: f32 = 4.0;

/// Configuration for drawing a focus ring.
#[derive(Debug, Clone, Copy)]
pub struct FocusRing {
    /// The color of the focus ring.
    pub color: Color,
    /// The width of the focus ring border.
    pub width: f32,
    /// The offset from the widget bounds (expands outward if positive).
    pub offset: f32,
    /// The border radius of the focus ring.
    pub radius: f32,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self {
            color: Color::from_rgb(0.4, 0.6, 1.0), // Default blue
            width: FOCUS_RING_WIDTH,
            offset: FOCUS_RING_OFFSET,
            radius: FOCUS_RING_RADIUS,
        }
    }
}

impl FocusRing {
    /// Creates a new focus ring with the given color.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// Creates a focus ring from the theme's accent focus color.
    pub fn from_theme(theme: &Theme) -> Self {
        Self::new(theme.accent.focus)
    }

    /// Sets the width of the focus ring.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the offset of the focus ring from the widget bounds.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the border radius of the focus ring.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Computes the bounds for the focus ring based on widget bounds.
    pub fn bounds(&self, widget_bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: widget_bounds.x - self.offset,
            y: widget_bounds.y - self.offset,
            width: widget_bounds.width + 2.0 * self.offset,
            height: widget_bounds.height + 2.0 * self.offset,
        }
    }

    /// Draws the focus ring around the given bounds.
    pub fn draw<Renderer>(&self, renderer: &mut Renderer, widget_bounds: Rectangle)
    where
        Renderer: renderer::Renderer,
    {
        let bounds = self.bounds(widget_bounds);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: self.color,
                    width: self.width,
                    radius: self.radius.into(),
                },
                ..Default::default()
            },
            Color::TRANSPARENT,
        );
    }
}

/// Draws a focus ring around the given bounds using the theme's accent focus color.
///
/// This is a convenience function for the common case.
pub fn draw_focus_ring<Renderer>(renderer: &mut Renderer, theme: &Theme, bounds: Rectangle)
where
    Renderer: renderer::Renderer,
{
    FocusRing::from_theme(theme).draw(renderer, bounds);
}

/// Draws a focus ring with custom border radius.
pub fn draw_focus_ring_with_radius<Renderer>(
    renderer: &mut Renderer,
    theme: &Theme,
    bounds: Rectangle,
    radius: f32,
) where
    Renderer: renderer::Renderer,
{
    FocusRing::from_theme(theme)
        .radius(radius)
        .draw(renderer, bounds);
}
