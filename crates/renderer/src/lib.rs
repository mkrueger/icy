//! The official renderer for iced.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "wgpu-bare")]
pub use icy_ui_wgpu as wgpu;

pub mod fallback;

pub use icy_ui_graphics as graphics;
pub use icy_ui_graphics::core;

#[cfg(feature = "geometry")]
pub use icy_ui_graphics::geometry;

/// The default graphics renderer for [`icy_ui`].
///
/// [`icy_ui`]: https://github.com/iced-rs/iced
pub type Renderer = renderer::Renderer;

/// The default graphics compositor for [`icy_ui`].
///
/// [`icy_ui`]: https://github.com/iced-rs/iced
pub type Compositor = renderer::Compositor;

#[cfg(all(feature = "wgpu-bare", feature = "tiny-skia"))]
mod renderer {
    pub type Renderer =
        crate::fallback::Renderer<icy_ui_wgpu::Renderer, icy_ui_tiny_skia::Renderer>;

    pub type Compositor = crate::fallback::Compositor<
        icy_ui_wgpu::window::Compositor,
        icy_ui_tiny_skia::window::Compositor,
    >;
}

#[cfg(all(feature = "wgpu-bare", not(feature = "tiny-skia")))]
mod renderer {
    pub type Renderer = icy_ui_wgpu::Renderer;
    pub type Compositor = icy_ui_wgpu::window::Compositor;
}

#[cfg(all(not(feature = "wgpu-bare"), feature = "tiny-skia"))]
mod renderer {
    pub type Renderer = icy_ui_tiny_skia::Renderer;
    pub type Compositor = icy_ui_tiny_skia::window::Compositor;
}

#[cfg(not(any(feature = "wgpu-bare", feature = "tiny-skia")))]
mod renderer {
    #[cfg(not(debug_assertions))]
    compile_error!(
        "Cannot compile `icy_ui_renderer` in release mode \
        without a renderer feature enabled. \
        Enable either the `wgpu` or `tiny-skia` feature, or both."
    );

    pub type Renderer = ();
    pub type Compositor = ();
}
