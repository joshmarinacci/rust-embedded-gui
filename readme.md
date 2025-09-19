# Untitled Embedded Rust GUI

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

Run the simulator example with `cargo run --example simulator --features std`.

Run the unit tests with `cargo test --features std`.



## Roadmap

[x] Remove generics for color and font. Just use embedded graphics directly.
[ ] use simulator for interactive tests
[ ] use MockDisplay for automated tests
[ ] setup CI on github actions.
[ ] add menu view
[ ] add list view
[ ] support layout using font size. needs padding in the widgets.



Make components size self in layout pass using current font size 

* Button and label sizes 
* Text input?
* How does form react to this?
* trigger revalidate layout on font size change and theme change
* Scene needs to invalidate layout Boolean
* Add mouse input to simulator 