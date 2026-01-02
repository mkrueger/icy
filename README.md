# Icy UI

Just a fork of https://github.com/iced-rs/iced

Which is IMHO a very good UI library. My tools have very specific needs that maybe no other APP has. 
Which are:

+ Running on linux, osx and windows natively
+ Empowering apps to have good platform integration while providing own UIs
   + Working clipboard handling of multiple formats
   + DND 
   + Native UI pradigms should work (like osx main menu or mnemonics)
+ And custom controls should be easier to implement (event system needs more data)

So I started that and added:

+ Mouse event handling with modifiers 
+ Clipboard handling that works and isn't just a lackluster
+ Scrollbar that's not from 1998 and works on large areas as well
+ Menus 
+ Extended theming (libcosmic <3)
+ Support for focus/keyboard input for all controls
+ Drag & drop support
+ Accessiblity

I was forced to fork it. I don't intend to break much away from iced to be able to port/take upstream changes.
Icy UI is just ment to be used by my own tool suite for now.

Which has very specific needs - see above. Best is to stay with iced or libcosmic for now. I don't have an idea
where this is going to but I need more out of an UI lib than what the existing ones deliver.

However you can take a look and if you have similiar needs feel free to use this fork instead.

I try to keep doc up 2 date. I take PRs and ideas and try to listen to my users instead of dictating things. 

I can't promise to take/implement all feature request because of my time limits. 

But the goal is to make a more community friendly project.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.