# Icy UI

Icy UI is a fork of https://github.com/iced-rs/iced.

Iced is a great UI library. This fork exists because some of my applications have fairly specific “desktop app” requirements that were hard to achieve cleanly without changes upstream.

## Goals

- Native apps on Linux, macOS, and Windows
- Better platform integration while still allowing fully custom UIs
   - Clipboard handling (including multiple formats; especially important on Wayland)
   - Drag & drop
   - Native UI paradigms where applicable (e.g. macOS main menu, mnemonics on Windows/Linux)
- Easier custom controls (more complete event data)

## What’s different from upstream

- Mouse event handling with modifiers
- Improved clipboard handling
- Scrollbar behavior that works on large scroll areas
- Menus (including mnemonics)
- Extended theming (inspired by libcosmic)
- Better focus / keyboard input support across controls
- Drag & drop support
- Accessibility 
- RTL support

## Status / scope

This is primarily used by my own tool suite right now.

I try to stay close to upstream iced to make it easier to pick up changes over time. If you do not need the features above, you probably want to use upstream iced (or libcosmic) instead.

If you have similar requirements, feel free to try this fork.

## Quick start

Build the workspace:

```sh
cargo build
```

Run some examples:

```sh
cd examples/demo_app && cargo run
```

```sh
cd examples/menu && cargo run
```

```sh
cd examples/focus && cargo run
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.