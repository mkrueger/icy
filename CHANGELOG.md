# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `icy_ui_macos` crate for macOS-specific platform integration
  - **URL Handler**: Register custom URL schemes (e.g., `myapp://action`) and receive URL events via `event::listen_url()`
  - **DnD Initiation**: Start native drag-and-drop operations from your app using `NSDraggingSource`
- Event Log page in demo app for debugging platform events

### Changed
- Switched from iced-rs/winit fork to vanilla winit 0.30.12 from crates.io
  - URL handling now implemented via `icy_ui_macos` instead of winit fork extensions
  - Removes dependency on forked winit, improving maintainability

