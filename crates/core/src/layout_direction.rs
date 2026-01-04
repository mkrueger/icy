//! Layout direction support for RTL (right-to-left) languages.
//!
//! This module provides the [`LayoutDirection`] enum and global state management
//! for bidirectional layout support (Arabic, Hebrew, Persian, etc.).
//!
//! # Global Layout Direction
//!
//! Similar to accessibility mode, there's a global layout direction that widgets
//! use as a fallback when they don't have an explicit direction set:
//!
//! ```rust,ignore
//! use icy_ui_core::{LayoutDirection, set_layout_direction, layout_direction};
//!
//! // Set global direction to RTL
//! set_layout_direction(LayoutDirection::Rtl);
//!
//! // Widgets without explicit direction use the global value
//! assert!(layout_direction().is_rtl());
//! ```
//!
//! # Per-Widget Override
//!
//! Widgets can override the global direction via `.layout_direction()`:
//!
//! ```rust,ignore
//! use icy_ui::{widget::menu_bar, LayoutDirection};
//!
//! // This menu bar is always RTL, regardless of global setting
//! menu_bar(items).layout_direction(LayoutDirection::Rtl)
//! ```

use std::sync::atomic::{AtomicU8, Ordering};

// ============================================================================
// Global Layout Direction State
// ============================================================================

/// Global layout direction state.
///
/// Stored as u8: 0 = Ltr, 1 = Rtl
static LAYOUT_DIRECTION: AtomicU8 = AtomicU8::new(0);

/// Returns the global layout direction.
///
/// Widgets use this as a fallback when they don't have an explicit direction set.
/// This is typically set by the shell based on system locale or user preference.
pub fn layout_direction() -> LayoutDirection {
    match LAYOUT_DIRECTION.load(Ordering::Relaxed) {
        1 => LayoutDirection::Rtl,
        _ => LayoutDirection::Ltr,
    }
}

/// Sets the global layout direction.
///
/// This should be called by the shell when the layout direction changes,
/// for example via `window::set_layout_direction()`.
pub fn set_layout_direction(direction: LayoutDirection) {
    LAYOUT_DIRECTION.store(direction as u8, Ordering::Relaxed);
}

// ============================================================================
// Layout Direction Enum
// ============================================================================

/// The direction of the layout flow.
///
/// This determines whether the layout flows from left-to-right (LTR)
/// or right-to-left (RTL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LayoutDirection {
    /// Left-to-right layout (default for most Western languages).
    #[default]
    Ltr = 0,
    /// Right-to-left layout (for Arabic, Hebrew, etc.).
    Rtl = 1,
}

impl LayoutDirection {
    /// Returns `true` if the layout direction is left-to-right.
    pub fn is_ltr(self) -> bool {
        matches!(self, Self::Ltr)
    }

    /// Returns `true` if the layout direction is right-to-left.
    pub fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }

    /// Returns the opposite layout direction.
    pub fn flip(self) -> Self {
        match self {
            Self::Ltr => Self::Rtl,
            Self::Rtl => Self::Ltr,
        }
    }

    /// Resolves a logical start/end position to a physical left/right position.
    ///
    /// In LTR: start = left, end = right
    /// In RTL: start = right, end = left
    pub fn resolve_start_end<T>(self, start: T, end: T) -> (T, T) {
        match self {
            Self::Ltr => (start, end),
            Self::Rtl => (end, start),
        }
    }
}
