# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- See doc/new what changed in 
  - clipboard, mouse events, scrollbars and theming.
  - got added new buttons, clolr/date packer, dnd, menus, toaster and accessibility
- New `icy_ui_macos` crate for macOS-specific platform integration
  - **URL Handler**: Register custom URL schemes (e.g., `myapp://action`) and receive URL events via `event::listen_url()`
  - **DnD Initiation**: Start native drag-and-drop operations from your app using `NSDraggingSource`
- New `icy_ui_windows` crate for Windows-specific platform integration
  - **DnD Initiation**: Start native drag-and-drop operations using OLE `IDropSource`/`IDataObject`
  - Supports text, file lists, and custom MIME types via COM interfaces
- Event Log page in demo app for debugging platform events
- New `overlay::modal::Modal` helper to present blocking modal dialogs via the overlay system (includes an About dialog in `demo_app` showcasing it)

### Changed
- Switched from iced-rs/winit fork to vanilla winit 0.30.12 from crates.io
  - URL handling now implemented via `icy_ui_macos` instead of winit fork extensions
  - Removes dependency on forked winit, improving maintainability

