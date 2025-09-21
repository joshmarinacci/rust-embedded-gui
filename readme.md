# Untitled Embedded Rust GUI

![screenshot](resources/screenshot-001.png)

## What is This?

This is a new GUI library for no_std embedded Rust. I currently have it running on
the ESP32-S3 based Lilygo T-Deck, but it should run on anything that uses embedded_graphics.
The library manages a scene of views and has built in components for:

* button
* label
* text input (text box)
* panel
* toggle button
* toggle group

Views are rendered using a Theme which can be customized for different
colors and font sizes.  Views carry their own internal state using an
optional state object. Application state should remain outside the scene/view structure
and be handled by processing actions emitted from the scene when events happen.

## Usage

Build the library with `cargo build`.

Run the simulator example with `cargo run --example simulator --features std`. Note that
the simulator needs SDL2. [Install instructions](https://docs.rs/embedded-graphics-simulator/latest/embedded_graphics_simulator/).



Run the unit tests with `cargo test --features std`.



## Roadmap

- [x] Remove generics for color and font. Just use embedded graphics directly.
- [x] use simulator for interactive tests
- [x] use MockDisplay for automated tests
- [x] support layout using font size. needs padding in the widgets.
- [ ] setup CI on github actions.
- [ ] add menu view
- [ ] add list view
- [x] add hbox and vbox layouts
- [ ] add tab panel
- [ ] make children drawn and picked relative to the parent.
  - [ ] calculating dirty rect needs to be converted back to global 


* cursor in the text input
* accent fg & bg colors for theme
* use form layout for the buttons panel
* don't double draw borders for panels inside of tab panel
* let tab panel define it's own children using a toggle group
* let tab panel switch it's own tabs instead of using external handle action
* add padding to view 
* add border boolean to view